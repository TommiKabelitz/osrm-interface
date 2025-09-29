#include <osrm/osrm.hpp>
#include <osrm/table_parameters.hpp>
#include <osrm/engine_config.hpp>
#include <osrm/json_container.hpp>
#include <osrm/route_parameters.hpp>
#include <osrm/trip_parameters.hpp>
#include "json/json_serialiser.cpp"
// #include <mapbox/variant.hpp>

#include <string>
#include <iostream>
#include <cstdlib>

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

    OSRM_Result osrm_table(void *osrm_instance,
                           const double *coordinates,
                           size_t num_coordinates,
                           const size_t *sources,
                           size_t num_sources,
                           const size_t *destinations,
                           size_t num_destinations)
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

    enum RouteFlags : uint8_t
    {
        ROUTE_ALTERNATIVES = 1 << 0,
        ROUTE_STEPS = 1 << 1,
        ROUTE_ANNOTATIONS = 1 << 2,
        ROUTE_CONTINUE_STRAIGHT = 1 << 3,
    };

    OSRM_Result osrm_route(void *osrm_instance,
                           const double *coordinates,
                           size_t num_coordinates,
                           enum GeometryType geometry_type,
                           enum OverviewZoom overview_zoom,
                           uint8_t flags)
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