#include <raylib.h>
#ifdef USE_EMBEDDED_IMAGES
    #include "res/img_h/flags.h"
    #include "res/img_h/arrows.h"
#endif
#include "list.hpp"
#include "extlib/picojson.hpp"
#include "ui/cursor.cpp"
#include <string>
#include <algorithm>
#include <fstream>
#include <chrono>

std::string current_time_hms()
{
    using namespace std::chrono;
    const auto now = system_clock::now();
    return std::format("{:%H:%M:%S}", floor<seconds>(now));
}

int main() {
    std::string filePath = "rayball_config.json";
    std::ifstream file(filePath);

    if (!file.is_open()) {
        TraceLog(LOG_WARNING, "Couldn't open configuration file or wasn't found! Writing a new one now.");
        std::ofstream out(filePath);
        out << defaultConfigFile;
        out.close();
    } else {
        std::string config;
        getline(file, config);
        picojson::value v;
        std::string err = picojson::parse(v, config);
        if (err.empty()) {
            const auto& obj = v.get<picojson::object>();
            Config::FancyCursor         = obj.at("FancyCursor").get<bool>();
            Config::ShowFlagImages      = obj.at("ShowFlagImages").get<bool>();
            Config::ScrollingBackground = obj.at("ScrollingBackground").get<bool>();
            Config::HideLocked          = obj.at("HideLocked").get<bool>();
            Config::FPS                 = obj.at("FPS").get<double>();
            Config::Longitude           = obj.at("Longitude").get<double>();
            Config::Latitude            = obj.at("Latitude").get<double>();
        } else {
            TraceLog(LOG_WARNING, "Couldn't parse configuration file! Please check that it is properly formatted.");
        }
    }

    InitWindow(1280, 720, "rayball");
    SetWindowState(FLAG_WINDOW_RESIZABLE);
    SetWindowMinSize(640, 360);
    Config::FancyCursor ? HideCursor() : ShowCursor();
    SetTargetFPS(Config::FPS);

    const char * room_api_link = "https://haxball.com/rs/api/list";
    auto rooms = HaxballParser::fetchRooms(room_api_link);

    std::sort(rooms.begin(), rooms.end(),
              [](const Room& a, const Room& b) {
                  return a.distance_km < b.distance_km;
            });

    Layout::playerCountWidth = MeasureText("WW/WW", Layout::fontSize); // have to account for widest
    Layout::distanceWidth = MeasureText("10000KM", Layout::fontSize); // ditto

    int width = 64;
    int height = 64;

    Color *pixels = (Color *)malloc(width*height*sizeof(Color));

    for (int y = 0; y < height; y++)
    {
        for (int x = 0; x < width; x++)
        {
            if (((x/32+y/32)/1)%2 == 0) pixels[y*width + x] = Style::bgColor1;
            else pixels[y*width + x] = Style::bgColor2;
        }
    }

    Image checkedIm = {
        .data = pixels,
        .width = width,
        .height = height,
        .mipmaps = 1,
        .format = PIXELFORMAT_UNCOMPRESSED_R8G8B8A8
    };

    Texture2D bg = LoadTextureFromImage(checkedIm);
    
    #ifdef USE_EMBEDDED_IMAGES
        const Image flag_image = LoadImageFromMemory(".png", flags_png, flags_png_len);
        Layout::flagTextureAtlas = LoadTextureFromImage(flag_image);
        UnloadImage(flag_image);

        const Image arrows_image = LoadImageFromMemory(".png", res_img_arrows_png, res_img_arrows_png_len);
        Layout::fonts[0] = LoadFontFromImage(arrows_image, MAGENTA, 32);
        UnloadImage(arrows_image);
    #else
        Layout::fonts[0] = LoadFont("res/img/arrows.png");
        Layout::flagTextureAtlas = LoadTexture("res/img/flags.png");
    #endif

    SetTextureFilter(bg, TEXTURE_FILTER_BILINEAR);
    SetTextureWrap(bg, TEXTURE_WRAP_REPEAT);

    int page = 0;
    int roomsPerPage = 22;
    float scroll = 0;
    
    Rectangle ShowFlagImagesToggleRect = {0, 48, 128, 24};

    std::vector<ToggleButton> settings = {
        {"Flags", Config::ShowFlagImages},
        {"Fancy Cursor", Config::FancyCursor},
        {"Scrolling BG", Config::ScrollingBackground},
        {"Hide Locked", Config::HideLocked},
    };

    std::vector<Button> list_actions = {
        {"!"},
        {"$"},
        {"\""}
    };

    std::vector<Button> navbarActions = {
        {"rayball"},
        {"server list"},
        {"settings"},
    };

    std::vector<int> navbarButtonSizes = {};

    for (int i = 0; i < navbarActions.size(); i++) {
        int textWidth = MeasureText(navbarActions[i].txt.c_str(), Layout::fontSize);
        navbarButtonSizes.push_back(textWidth);
    }
    int list_width = 400;
    int windowWidth = GetRenderWidth();
    int windowHeight = GetRenderHeight();
    int currentScreen = ServerList;

    while (!WindowShouldClose()) {
        float dt = GetFrameTime();
        windowWidth = GetRenderWidth();
        windowHeight = GetRenderHeight();

        BeginDrawing();
        ClearBackground(BLACK);

        if (Config::ScrollingBackground) {
            scroll -= 25 * dt;
            if (scroll <= -bg.width) scroll = 0;
        }

        // NOTE: ALWAYS SET THE COLOR TO WHITE WHEN DRAWING TEXTURES!!!
        DrawTextureRec(bg, (Rectangle){ scroll, 0, (float)GetScreenWidth(), (float)GetScreenHeight() }, (Vector2){0,0}, WHITE);

        DrawRectangleRec((Rectangle){ 0, 0, (float)GetScreenWidth(), (float)Layout::buttonHeight}, Style::secondaryColor);
        DrawText("rayball", Layout::spacing, Layout::spacing, Layout::fontSize, WHITE);
        auto HMSTimeString = current_time_hms();
        auto HMSTimeWidth = MeasureText(HMSTimeString.c_str(), Layout::fontSize);
        DrawText(HMSTimeString.c_str(), windowWidth-Layout::spacing-HMSTimeWidth, Layout::spacing, Layout::fontSize, WHITE);

        for (int i = 0; i < navbarActions.size(); i++) {
            Rectangle rect;
            bool active;
            switch (i) {
                case RayballLogo:
                    rect = {(float)0,0,(float)navbarButtonSizes[RayballLogo],(float)Layout::buttonHeight};
                    active = navbarActions[i].Draw(rect);
                    if (active) system("xdg-open https://github.com/stuxvii/rayball");
                    break;
                case ServerList:
                    rect = {(float)navbarButtonSizes[RayballLogo]+Layout::spacing,0,(float)navbarButtonSizes[ServerList]+Layout::spacing,(float)Layout::buttonHeight};
                    active = navbarActions[i].Draw(rect);
                    if (active) currentScreen = ServerList;
                break;
                case Configuration:
                    rect = {(float)navbarButtonSizes[RayballLogo]+navbarButtonSizes[ServerList]+(Layout::spacing*i),0,(float)navbarButtonSizes[Configuration],(float)Layout::buttonHeight};
                    active = navbarActions[i].Draw(rect);
                    if (active) currentScreen = Configuration;
                break;
            }
        }

        switch (currentScreen) {
            case Configuration:
                for (int i = 0; i < settings.size(); i++) {
                    int configWindowX = windowWidth / 2;
                    int configWindowY = windowHeight / 2;
                    configWindowY -= Layout::buttonHeight / settings.size();
                    configWindowX /= 1.25;

                    Rectangle this_rect = {
                        (float)configWindowX,
                        (float)configWindowY + (i * Layout::buttonHeight),
                        (float)configWindowX/2,
                        (float)Layout::buttonHeight,
                    };

                    bool button_was_toggled = settings[i].isToggled;
                    bool state = settings[i].Draw(this_rect);
                    if (button_was_toggled != state) {
                        switch (i) {
                            case ShowFlags:
                                Config::ShowFlagImages = state;
                                break;
                            case UseFancyCursor:
                                Config::FancyCursor = state;
                                state ? HideCursor() : ShowCursor();
                                break;
                            case ScrollingBG:
                                Config::ScrollingBackground = state;
                                break;
                            case HideLocked:
                                Config::HideLocked = state;
                                break;
                        }
                    }
                }
            break;
            case ServerList:
                int roomListX = windowWidth / 2;
                int roomListY = windowHeight / 2;
                roomListX -= list_width / 2;
                roomListX -= Layout::flagSize.x;
                roomListY -= (Layout::buttonHeight * roomsPerPage) / 2;
                roomListY += Layout::buttonHeight;

                int visibleIndex = 0;

                for (int i = (roomsPerPage * page); i < (roomsPerPage * (page+1)); i++) {
                    if (static_cast<size_t>(i) >= rooms.size()) break;
                    if (Config::HideLocked && rooms[i].locked) continue;
                    Rectangle rect = {
                        static_cast<float>(roomListX),
                        (float)roomListY + visibleIndex * Layout::buttonHeight,
                        static_cast<float>(list_width-64),
                        Layout::fontSize
                    };
                    rooms[i].Draw(rect);
                    visibleIndex++;
                }

                for (int i = 0; i < list_actions.size(); i++) {
                    Rectangle rect = {static_cast<float>(roomListX-Layout::fontSize-Layout::spacing), static_cast<float>(i*Layout::buttonHeight)+roomListY, Layout::fontSize, Layout::fontSize};
                    bool active;
                    switch (i) {
                        case NextPage:
                            if (page + 1 > rooms.size() / roomsPerPage) break;
                            active = list_actions[i].DrawFont(rect,Layout::fonts[0]);
                            if (active) page++;
                        break;
                        case LastPage:
                            if (page < 1) break;
                            active = list_actions[i].DrawFont(rect,Layout::fonts[0]);
                            if (active) page--;
                        break;
                        case Refresh:
                            active = list_actions[i].DrawFont(rect,Layout::fonts[0]);
                            if (active)
                            {
                                rooms = HaxballParser::fetchRooms(room_api_link);
                                std::sort(rooms.begin(), rooms.end(),
                                      [](const Room& a, const Room& b) {
                                          return a.distance_km < b.distance_km;
                                      });
                            }
                        break;
                    }
                }
            break;
        }

        std::string fps = std::to_string(GetFPS());
        DrawRectangle(0, windowHeight-Layout::fontSize, MeasureText(fps.c_str(), Layout::fontSize), Layout::fontSize, Color{0,0,0, 127});
        DrawText(fps.c_str(), 0, windowHeight-Layout::fontSize, Layout::fontSize, GREEN);

        if (Config::FancyCursor) DrawCursorTrail(dt);
        EndDrawing();
    }

    curl_global_cleanup();
    CloseWindow();

    return 0;
}
