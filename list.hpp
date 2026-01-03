#pragma once

#include "ui/primitives.hpp"
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <curl/curl.h>
#include <raylib.h>
#include <string>
#include <vector>
#include <format>

inline float read_float(const std::vector<uint8_t>& data, size_t pos) {
    if (pos + 4 > data.size()) return 0.0f;
    float f;
    std::memcpy(&f, &data[pos], 4);
    return f;
}

class HaxballParser {
  static size_t WriteCallback(void *contents, size_t size, size_t nmemb,
                              std::vector<uint8_t> *userp) {
    userp->insert(userp->end(), (uint8_t *)contents,
                  (uint8_t *)contents + size * nmemb);
    return size * nmemb;
  }

public:
  static std::vector<Room> fetchRooms(const std::string &url) {
    std::vector<uint8_t> buf;
    std::vector<Room> rooms;
    CURL *curl = curl_easy_init();

    if (curl) {
      curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
      curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
      curl_easy_setopt(curl, CURLOPT_WRITEDATA, &buf);
      curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);

      if (curl_easy_perform(curl) == CURLE_OK && !buf.empty()) {
        size_t pos = 1; // needed for data to parse correctly
        while (pos + 1 < buf.size()) {
          if (buf[pos++] != 0x00)
            break;

          uint8_t id_len = buf[pos++];
          std::string id((char *)&buf[pos], id_len);
          pos += id_len + 1;

          uint8_t block_len = buf[pos++];
          size_t next_entry = pos + block_len;

          pos += 2;
          
          uint8_t name_len = buf[pos++];
          std::string name((char *)&buf[pos], name_len);
          pos += name_len;

          uint8_t c_len = buf[pos++];
          std::string country((char *)&buf[pos], c_len);
          pos += c_len;

          float x = read_float(buf, pos);
          pos += 4;
          float y = read_float(buf, pos);
          pos += 4;

          Vector2 flag_coords;

          if (flag_map.contains(country)) {
            flag_coords = flag_map.at(country);
          } else {
            flag_coords = Vector2 {-80, -198};
          }

          int flagTAtlasWidth = Layout::flagTextureAtlas.width;
          int flagTAtlasHeight = Layout::flagTextureAtlas.height;

          Rectangle flagsRec = {
              flagTAtlasWidth - flag_coords.x, //offset fix
              flagTAtlasHeight - flag_coords.y, 
              (float)16, (float)11};

          bool locked = buf[pos++];

          uint8_t max_players = buf[pos++];
          uint8_t players = buf[pos++];

          std::string a = std::to_string(players);
          std::string b = std::to_string(max_players);
          std::string label = std::format("{}/{}", a, b);

          rooms.emplace_back(Room{name, id, country, Vector2{x,y}, a.c_str(), b.c_str(), flagsRec, label, locked});
          pos = next_entry;
        }
        curl_easy_cleanup(curl);
      } else {
        TraceLog(LOG_INFO, "Couldn't form a connection to HaxBall's servers!");
        rooms.emplace_back(Room{"user_offline", "You're offline!", "na", Vector2{0,0},"","", Rectangle{0,0,0,0}, "", true});
      }
    }
    return rooms;
  }
};
