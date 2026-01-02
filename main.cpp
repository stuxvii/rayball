#include <raylib.h>
#ifdef USE_EMBEDDED_IMAGES
    #include "res/img_h/flags.h"
    #include "res/img_h/arrows.h"
#endif
#include "list.hpp"
#include "ui/cursor.cpp"
#include <string>

void join(Room room) {
    TraceLog(LOG_INFO, "%s, %s", room.players, room.max_players);
    TraceLog(LOG_INFO, "%s, %s", std::to_string(room.coords.x).c_str(), std::to_string(room.coords.y).c_str());
    TraceLog(LOG_INFO, "%s, %s", room.id.c_str(), room.country.c_str());
}

int main() {
    InitWindow(640, 480, "trail");
    HideCursor();
    SetTargetFPS(FPS);

    const char * room_api_link = "https://html5.haxball.com/rs/api/list";
    auto rooms = HaxballParser::fetchRooms(room_api_link);
    
    int width = 64;
    int height = 64;

    Color *pixels = (Color *)malloc(width*height*sizeof(Color));

    for (int y = 0; y < height; y++)
    {
        for (int x = 0; x < width; x++)
        {
            if (((x/32+y/32)/1)%2 == 0) pixels[y*width + x] = Color{127, 170, 127, 255};
            else pixels[y*width + x] = Color{0, 127, 0, 255};
        }
    }

    Image checkedIm = {
        .data = pixels,
        .width = width,
        .height = height,
        .mipmaps = 1,
        .format = PIXELFORMAT_UNCOMPRESSED_R8G8B8A8
    };

    Font fonts[8*8] = { 0 };

    Texture2D bg = LoadTextureFromImage(checkedIm);
    
    #ifdef USE_EMBEDDED_IMAGES
        const Image flag_image = LoadImageFromMemory(".png", flags_png, flags_png_len);
        flagTextureAtlas = LoadTextureFromImage(flag_image);
        UnloadImage(flag_image);

        const Image arrows_image = LoadImageFromMemory(".png", res_img_arrows_png, res_img_arrows_png_len);
        fonts[0] = LoadFontFromImage(arrows_image, MAGENTA, 32);
        UnloadImage(arrows_image);
    #else
        fonts[0] = LoadFont("res/img/arrows.png");
        flagTextureAtlas = LoadTexture("res/img/flags.png");
    #endif

    SetTextureFilter(bg, TEXTURE_FILTER_BILINEAR);
    SetTextureWrap(bg, TEXTURE_WRAP_REPEAT);

    int page = 0;
    int roomsPerPage = 30;
    float scroll = 0;
    
    Rectangle ShowFlagImagesToggleRect = {0, 48, 128, 24};

    std::vector<ToggleButton> settings = {
        {"FPS Limit", true},
        {"Flags", true},
        {"Fancy Cursor", true}
    };

    Button next_entry = {};
    next_entry.txt = "$";

    Button last_entry = {};
    last_entry.txt = "!";

    Button refresh = {};
    refresh.txt = "\"";

    while (!WindowShouldClose()) {
        float dt = GetFrameTime();
        float fontSizeFloat = fontSize;
        BeginDrawing();
        ClearBackground(BLACK);

        scroll -= 25 * dt;
        if (scroll <= -bg.width) scroll = 0;
        DrawTextureRec(bg, (Rectangle){ scroll, 0, (float)GetScreenWidth(), (float)GetScreenHeight() }, (Vector2){0,0}, WHITE);

        for (int i = 0; i < settings.size(); i++) {
            Rectangle this_rect = {0, static_cast<float>(32*i), 128, 24};
            bool button_was_toggled = settings[i].isToggled;
            bool state = settings[i].Draw(this_rect);
            switch (i) {
                case ShowFlags:
                    ShowFlagImages = state;
                break;
                case UnlockFPS:
                    if (button_was_toggled != state) {
                        FPS = state ? 240 : 0;
                        SetTargetFPS(FPS);
                    }
                break;
                case UseFancyCursor:
                    if (button_was_toggled != state) {
                        FancyCursor = state;
                        if (FancyCursor) {
                            HideCursor();
                        } else {
                            ShowCursor();
                        }
                    }
                break;
            }
        }

        int list_width = 400;
        int x_pos = GetRenderWidth() - list_width;
        int y_pos = GetRenderHeight();

        for (int i = (roomsPerPage * page); i < (roomsPerPage * (page+1)); i++) {
            if (static_cast<size_t>(i) >= rooms.size()) break;
            Rectangle rect = {
                static_cast<float>(x_pos), (float)(i - (page * roomsPerPage)) * (fontSize+2), static_cast<float>(list_width-64), fontSizeFloat
            };
            if (rooms[i].Draw(rect)) {
                //join(rooms[i]);
            }
        }

        if (page > 0) {
            Rectangle rect = {static_cast<float>(x_pos-34), 0, fontSizeFloat, fontSizeFloat};
            if (last_entry.DrawFont(rect,fonts[0])) {
                page--;
            }
        }

        if (page < rooms.size() / roomsPerPage) {
            Rectangle rect = {static_cast<float>(x_pos-34), fontSizeFloat+2, fontSizeFloat, fontSizeFloat};
            if (next_entry.DrawFont(rect,fonts[0])) {
                page++;
            }
        }

        if (rooms[0].id != "user_offline") {
            Rectangle rect = {static_cast<float>(x_pos-34), (fontSizeFloat*2)+4, fontSizeFloat, fontSizeFloat};
            if (refresh.DrawFont(rect,fonts[0])) {
                rooms = HaxballParser::fetchRooms(room_api_link);
            }
        }

        std::string fps = std::to_string(GetFPS());
        DrawRectangle(0, y_pos-fontSize, MeasureText(fps.c_str(), fontSize), fontSize, Color{0,0,0, 127});
        DrawText(fps.c_str(), 0, y_pos-fontSize, fontSize, GREEN);

        if (FancyCursor) DrawCursorTrail(dt);
        EndDrawing();
    }

    curl_global_cleanup();
    CloseWindow();

    return 0;
}
