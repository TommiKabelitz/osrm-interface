#include <osrm/osrm.hpp>
#include <osrm/table_parameters.hpp>
#include <osrm/engine_config.hpp>
#include <osrm/json_container.hpp>
#include <osrm/route_parameters.hpp>
#include <osrm/match_parameters.hpp>
#include <osrm/trip_parameters.hpp>
#include <osrm/nearest_parameters.hpp>
#include "json/json_serialiser.cpp"

#include <string>
#include <iostream>
#include <cstdlib>

// Should not extern uint8_t enums as the C ABI is different
enum RouteFlags : uint8_t
{
    ROUTE_ALTERNATIVES = 1 << 0,
    ROUTE_STEPS = 1 << 1,
    ROUTE_ANNOTATIONS = 1 << 2,
    ROUTE_CONTINUE_STRAIGHT = 1 << 3,
};

enum MatchFlags : uint8_t
{
    MATCH_TIDY = 1 << 0,
    MATCH_STEPS = 1 << 1,
    MATCH_ANNOTATIONS = 1 << 2,
    MATCH_GENERATE_HINTS = 1 << 3,
};

struct ArrayString
{
    size_t len;
    const uint8_t *pointer;
};

extern "C"
{

    static thread_local std::string last_error;

    enum GeometryType
    {
        GeometryType_Polyline = 0,
        GeometryType_Polyline6 = 1,
        GeometryType_GeoJSON = 2,
    };

    enum OverviewZoom
    {
        OverviewZoom_Simplified = 0,
        OverviewZoom_Full = 1,
        OverviewZoom_False = 2,
    };

    struct OSRM_Result
    {
        int code;
        char *message;
    };

    void *osrm_create(const char *base_path, const char *algorithm)
    {
        try
        {
            osrm::EngineConfig config;
            config.storage_config = {base_path};
            config.use_shared_memory = false;

            if (strcmp(algorithm, "CH") == 0)
            {
                config.algorithm = osrm::EngineConfig::Algorithm::CH;
            }
            else if (strcmp(algorithm, "MLD") == 0)
            {
                config.algorithm = osrm::EngineConfig::Algorithm::MLD;
            }
            else
            {
                config.algorithm = osrm::EngineConfig::Algorithm::MLD;
            }

            return new osrm::OSRM(config);
        }
        catch (const std::exception &e)
        {
            last_error = e.what();
            return nullptr;
        }
    }

    void osrm_destroy(void *osrm_instance)
    {
        if (osrm_instance)
        {
            delete static_cast<osrm::OSRM *>(osrm_instance);
        }
    }

    enum class FallbackCoordinateType
    {
        FallbackCoordinateType_Input = 0,
        FallbackCoordinateType_Snapped = 1
    };

    enum class AnnotationsType
    {
        AnnotationsType_None = 0,
        AnnotationsType_Duration = 0x01,
        AnnotationsType_Distance = 0x02,
        AnnotationsType_All = AnnotationsType_Duration | AnnotationsType_Distance
    };

    OSRM_Result osrm_table(void *osrm_instance,
                           const double *coordinates,
                           size_t num_coordinates,
                           const size_t *sources,
                           size_t num_sources,
                           const size_t *destinations,
                           size_t num_destinations,
                           enum AnnotationsType annotations,
                           double fallback_speed,
                           enum FallbackCoordinateType fallback_coordinate_type,
                           double scale_factor)
    {

        if (!osrm_instance)
        {
            const char *err = "OSRM instance not found";
            char *msg = new char[strlen(err) + 1];
            strcpy(msg, err);
            return {1, msg};
        }

        osrm::OSRM *osrm_ptr = static_cast<osrm::OSRM *>(osrm_instance);
        osrm::TableParameters params;

        for (size_t i = 0; i < num_coordinates; ++i)
        {
            params.coordinates.push_back({osrm::util::FloatLongitude{coordinates[i * 2]},
                                          osrm::util::FloatLatitude{coordinates[i * 2 + 1]}});
        }

        if (num_sources > 0)
        {
            params.sources.assign(sources, sources + num_sources);
        }

        if (num_destinations > 0)
        {
            params.destinations.assign(destinations, destinations + num_destinations);
        }

        params.annotations = static_cast<osrm::engine::api::TableParameters::AnnotationsType>(annotations);
        if (fallback_speed > 0)
        {
            params.fallback_coordinate_type = static_cast<osrm::engine::api::TableParameters::FallbackCoordinateType>(fallback_coordinate_type);
            params.fallback_speed = fallback_speed;
        }
        if (scale_factor > 0)
        {
            params.scale_factor = scale_factor;
        }

        osrm::json::Object result;
        const auto status = osrm_ptr->Table(params, result);

        std::ostringstream oss;
        std::string result_str;
        int code;

        if (status == osrm::Status::Ok)
        {
            code = 0;
            serialize_object(oss, result);
            result_str = oss.str();
        }
        else
        {
            code = 1;
            try
            {
                result_str = std::get<osrm::util::json::String>(result.values.at("message")).value;
            }
            catch (const std::exception &e)
            {
                result_str = "Unknown OSRM error";
            }
        }

        char *message = new char[result_str.length() + 1];
        strcpy(message, result_str.c_str());

        return {code, message};
    }

    OSRM_Result osrm_route(void *osrm_instance,
                           const double *coordinates,
                           size_t num_coordinates,
                           enum GeometryType geometry_type,
                           enum OverviewZoom overview_zoom,
                           uint8_t flags, const ArrayString *excludes,
                           size_t num_excludes)
    {
        if (!osrm_instance)
        {
            const char *err = "OSRM instance not found";
            char *msg = new char[strlen(err) + 1];
            strcpy(msg, err);
            return {1, msg};
        }

        osrm::OSRM *osrm_ptr = static_cast<osrm::OSRM *>(osrm_instance);
        osrm::RouteParameters params;

        for (size_t i = 0; i < num_coordinates; ++i)
        {
            params.coordinates.push_back({osrm::util::FloatLongitude{coordinates[i * 2]},
                                          osrm::util::FloatLatitude{coordinates[i * 2 + 1]}});
        }

        params.geometries = static_cast<osrm::engine::api::RouteParameters::GeometriesType>(geometry_type);
        params.overview = static_cast<osrm::engine::api::RouteParameters::OverviewType>(overview_zoom);
        params.alternatives = (flags & ROUTE_ALTERNATIVES) != 0;
        params.steps = (flags & ROUTE_STEPS) != 0;
        params.annotations = (flags & ROUTE_ANNOTATIONS) != 0;
        params.continue_straight = (flags & ROUTE_CONTINUE_STRAIGHT) != 0;
        if (num_excludes > 0)
        {
            params.exclude.reserve(num_excludes);
            for (size_t i = 0; i < num_excludes; i++)
            {
                const ArrayString &e = excludes[i];

                if (e.pointer == nullptr || e.len == 0)
                {
                    continue;
                }

                std::string exclude_str(reinterpret_cast<const char *>(e.pointer), e.len);

                osrm::engine::Hint hint;
                params.exclude.push_back(std::move(exclude_str));
            }
        }
        osrm::json::Object result;
        const auto status = osrm_ptr->Route(params, result);

        std::ostringstream oss;
        std::string result_str;
        int code;

        if (status == osrm::Status::Ok)
        {
            code = 0;
            serialize_object(oss, result);
            result_str = oss.str();
        }
        else
        {
            code = 1;
            try
            {
                result_str = std::get<osrm::util::json::String>(result.values.at("message")).value;
            }
            catch (const std::exception &e)
            {
                result_str = "Unknown OSRM error";
            }
        }

        char *message = new char[result_str.length() + 1];
        strcpy(message, result_str.c_str());

        return {code, message};
    }

    enum class GapsType
    {
        Split = 0,
        Ignore = 1
    };

    OSRM_Result osrm_match(void *osrm_instance,
                           const double *coordinates,
                           size_t num_coordinates,
                           enum GeometryType geometry_type,
                           enum OverviewZoom overview_zoom,
                           const uint64_t *timestamps,
                           size_t num_timestamps,
                           enum GapsType gaps_type,
                           const size_t *waypoints,
                           size_t num_waypoints,
                           uint8_t flags,
                           const osrm::engine::Bearing **bearings,
                           size_t num_bearings,
                           const double *radiuses,
                           size_t num_radiuses,
                           const ArrayString *hints,
                           size_t num_hints,
                           const osrm::engine::Approach *approaches,
                           size_t num_approaches,
                           const ArrayString *excludes,
                           size_t num_excludes)
    {
        if (!osrm_instance)
        {
            const char *err = "OSRM instance not found";
            char *msg = new char[strlen(err) + 1];
            strcpy(msg, err);
            return {1, msg};
        }

        osrm::OSRM *osrm_ptr = static_cast<osrm::OSRM *>(osrm_instance);
        osrm::MatchParameters params;

        for (size_t i = 0; i < num_coordinates; ++i)
        {
            params.coordinates.push_back({osrm::util::FloatLongitude{coordinates[i * 2]},
                                          osrm::util::FloatLatitude{coordinates[i * 2 + 1]}});
        }

        params.geometries = static_cast<osrm::engine::api::RouteParameters::GeometriesType>(geometry_type);
        params.overview = static_cast<osrm::engine::api::RouteParameters::OverviewType>(overview_zoom);
        params.gaps = static_cast<osrm::engine::api::MatchParameters::GapsType>(gaps_type);
        params.tidy = (flags & MATCH_TIDY) != 0;
        params.steps = (flags & MATCH_STEPS) != 0;
        params.annotations = (flags & MATCH_ANNOTATIONS) != 0;
        if (num_timestamps > 0)
        {
            if (num_timestamps != num_coordinates)
            {
                const char *err = "num_timestamps must equal num_coordinates";
                char *msg = new char[strlen(err) + 1];
                strcpy(msg, err);
                return {1, msg};
            }
            params.timestamps.reserve(num_timestamps);
            for (size_t i = 0; i < num_timestamps; i++)
            {
                params.timestamps.push_back(timestamps[i]);
            }
        }
        if (num_waypoints > 0)
        {
            params.waypoints.reserve(num_waypoints);
            for (size_t i = 0; i < num_waypoints; i++)
            {
                params.waypoints.push_back(waypoints[i]);
            }
        }
        if (num_bearings > 0)
        {
            if (num_bearings != num_coordinates)
            {
                const char *err = "num_bearings must equal num_coordinates";
                char *msg = new char[strlen(err) + 1];
                strcpy(msg, err);
                return {1, msg};
            }
            params.bearings.reserve(num_bearings);
            for (size_t i = 0; i < num_bearings; i++)
            {
                params.bearings.push_back(*bearings[i]);
            }
        }

        if (num_radiuses > 0)
        {
            if (num_radiuses != num_coordinates)
            {
                const char *err = "num_radiuses must equal num_coordinates";
                char *msg = new char[strlen(err) + 1];
                strcpy(msg, err);
                return {1, msg};
            }
            params.radiuses.reserve(num_radiuses);
            for (size_t i = 0; i < num_radiuses; i++)
            {
                if (std::isinf(radiuses[i]))
                {
                    params.radiuses.push_back(std::nullopt);
                }
                else
                {
                    params.radiuses.push_back(radiuses[i]);
                }
            }
        }
        if (num_hints > 0)
        {
            if (num_hints != num_coordinates)
            {
                const char *err = "num_hints must equal num_coordinates";
                char *msg = new char[strlen(err) + 1];
                strcpy(msg, err);
                return {1, msg};
            }
            params.radiuses.reserve(num_hints);
            for (size_t i = 0; i < num_hints; i++)
            {
                const ArrayString &h = hints[i];

                if (h.pointer == nullptr || h.len == 0)
                {
                    params.hints.emplace_back();
                    continue;
                }

                std::string encoded_hint(reinterpret_cast<const char *>(h.pointer), h.len);

                osrm::engine::Hint hint;
                hint.FromBase64(encoded_hint);
                params.hints.push_back(std::move(hint));
            }
        }
        if (num_approaches > 0)
        {
            if (num_approaches != num_coordinates)
            {
                const char *err = "num_approaches must equal num_coordinates";
                char *msg = new char[strlen(err) + 1];
                strcpy(msg, err);
                return {1, msg};
            }
            params.approaches.reserve(num_approaches);
            for (size_t i = 0; i < num_approaches; i++)
            {
                params.approaches.push_back(approaches[i]);
            }
        }
        if (num_excludes > 0)
        {
            params.exclude.reserve(num_excludes);
            for (size_t i = 0; i < num_excludes; i++)
            {
                const ArrayString &e = excludes[i];

                if (e.pointer == nullptr || e.len == 0)
                {
                    continue;
                }

                std::string exclude_str(reinterpret_cast<const char *>(e.pointer), e.len);

                osrm::engine::Hint hint;
                params.exclude.push_back(std::move(exclude_str));
            }
        }
        osrm::json::Object result;
        const auto status = osrm_ptr->Match(params, result);

        std::ostringstream oss;
        std::string result_str;
        int code;

        if (status == osrm::Status::Ok)
        {
            code = 0;
            serialize_object(oss, result);
            result_str = oss.str();
        }
        else
        {
            code = 1;
            try
            {
                result_str = std::get<osrm::util::json::String>(result.values.at("message")).value;
            }
            catch (const std::exception &e)
            {
                result_str = "Unknown OSRM error";
            }
        }

        char *message = new char[result_str.length() + 1];
        strcpy(message, result_str.c_str());

        return {code, message};
    }

    OSRM_Result osrm_trip(void *osrm_instance,
                          const double *coordinates,
                          size_t num_coordinates)
    {

        if (!osrm_instance)
        {
            const char *err = "OSRM instance not found";
            char *msg = new char[strlen(err) + 1];
            strcpy(msg, err);
            return {1, msg};
        }

        osrm::OSRM *osrm_ptr = static_cast<osrm::OSRM *>(osrm_instance);
        osrm::TripParameters params;

        for (size_t i = 0; i < num_coordinates; ++i)
        {
            params.coordinates.push_back({osrm::util::FloatLongitude{coordinates[i * 2]},
                                          osrm::util::FloatLatitude{coordinates[i * 2 + 1]}});
        }

        osrm::json::Object result;
        const auto status = osrm_ptr->Trip(params, result);

        std::ostringstream oss;
        std::string result_str;
        int code;

        if (status == osrm::Status::Ok)
        {
            code = 0;
            serialize_object(oss, result);
            result_str = oss.str();
        }
        else
        {
            code = 1;
            try
            {
                result_str = std::get<osrm::util::json::String>(result.values.at("message")).value;
            }
            catch (const std::exception &e)
            {
                result_str = "Unknown OSRM error";
            }
        }

        char *message = new char[result_str.length() + 1];
        strcpy(message, result_str.c_str());

        return {code, message};
    }

    OSRM_Result osrm_nearest(void *osrm_instance,
                             const double longitude,
                             const double latitude,
                             uint64_t num_coordinates)
    {

        if (!osrm_instance)
        {
            const char *err = "OSRM instance not found";
            char *msg = new char[strlen(err) + 1];
            strcpy(msg, err);
            return {1, msg};
        }

        osrm::OSRM *osrm_ptr = static_cast<osrm::OSRM *>(osrm_instance);
        osrm::NearestParameters params;

        params.number_of_results = num_coordinates;
        params.coordinates.push_back({osrm::util::FloatLongitude{longitude},
                                      osrm::util::FloatLatitude{latitude}});

        osrm::json::Object result;
        const auto status = osrm_ptr->Nearest(params, result);

        std::ostringstream oss;
        std::string result_str;
        int code;

        if (status == osrm::Status::Ok)
        {
            code = 0;
            serialize_object(oss, result);
            result_str = oss.str();
        }
        else
        {
            code = 1;
            try
            {
                result_str = std::get<osrm::util::json::String>(result.values.at("message")).value;
            }
            catch (const std::exception &e)
            {
                result_str = "Unknown OSRM error";
            }
        }

        char *message = new char[result_str.length() + 1];
        strcpy(message, result_str.c_str());

        return {code, message};
    }

    const char *osrm_last_error()
    {
        return last_error.empty() ? nullptr : last_error.c_str();
    }

    void osrm_free_string(char *s)
    {
        if (s)
        {
            delete[] s;
        }
    }
}