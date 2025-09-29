#include <sstream>
#include <osrm/osrm.hpp>
#include "osrm/json_container.hpp"

#include <iomanip> // for std::setw, std::setfill

void handle_json_string_escapes(std::ostringstream &oss, const std::string &input)
{
    for (unsigned char c : input)
    {
        switch (c)
        {
        case '\"':
            oss << "\\\"";
            break;
        case '\\':
            oss << "\\\\";
            break;
        case '\b':
            oss << "\\b";
            break;
        case '\f':
            oss << "\\f";
            break;
        case '\n':
            oss << "\\n";
            break;
        case '\r':
            oss << "\\r";
            break;
        case '\t':
            oss << "\\t";
            break;
        default:
            if (c < 0x20)
            {
                // control chars -> \u00XX
                oss << "\\u"
                    << std::hex << std::setw(4) << std::setfill('0') << (int)c
                    << std::dec; // reset back to decimal
            }
            else
            {
                oss << c;
            }
        }
    }
}

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
        oss << "\"";
        handle_json_string_escapes(oss, p->value);
        oss << "\"";
    }
    else if (auto p = std::get_if<osrm::json::Number>(&v))
    {
        oss << p->value;
    }
    else if (std::get_if<osrm::json::True>(&v))
    {
        oss << "true";
    }
    else if (std::get_if<osrm::json::False>(&v))
    {
        oss << "false";
    }
    else if (std::get_if<osrm::json::Null>(&v))
    {
        oss << "null";
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
