#include <sstream>
#include <osrm/osrm.hpp>
#include "osrm/json_container.hpp"

void serialize_value(std::ostringstream &oss, const osrm::json::Value &v);

void serialize_object(std::ostringstream &oss, const osrm::json::Object &obj)
{
    oss << "{";
    bool first = true;
    for (const auto &[key, value] : obj.values)
    {
        if (!first)
            oss << ",";
        first = false;
        oss << "\"" << key << "\":";
        serialize_value(oss, value);
    }
    oss << "}";
}

void serialize_array(std::ostringstream &oss, const osrm::json::Array &arr)
{
    oss << "[";
    bool first = true;
    for (const auto &v : arr.values)
    {
        if (!first)
            oss << ",";
        first = false;
        serialize_value(oss, v);
    }
    oss << "]";
}

void serialize_value(std::ostringstream &oss, const osrm::json::Value &v)
{
    if (auto p = std::get_if<osrm::json::String>(&v))
    {
        oss << "\"" << p->value << "\"";
    }
    else if (auto p = std::get_if<osrm::json::Number>(&v))
    {
        oss << p->value;
    }
    else if (auto p = std::get_if<osrm::json::Boolean>(&v))
    {
        oss << (p->value ? "true" : "false");
    }
    else if (auto p = std::get_if<osrm::json::Array>(&v))
    {
        serialize_array(oss, *p);
    }
    else if (auto p = std::get_if<osrm::json::Object>(&v))
    {
        serialize_object(oss, *p);
    }
}
