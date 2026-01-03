#include "raylib.h"
#include <string>

const std::string defaultConfigFile = "{\"FancyCursor\": true,\"ShowFlagImages\": true,\"ScrollingBackground\": true,\"FPS\": 240,\"Longitude\": 0.0,\"Latitude\": 0.0}";

namespace Config {
    inline bool ShowFlagImages = false;
    inline bool FancyCursor = true;
    inline bool ScrollingBackground = true;
    inline int FPS = 240;
    inline int MaxFPS = 240;
    inline float Latitude = 0.0f;
    inline float Longitude = 0.0f;
}

namespace Layout {
    inline float fontSize = 13;
    inline int spacing = 1;
    inline int buttonHeight = fontSize + spacing;
    inline Texture2D flagTextureAtlas;
    inline Vector2 flagSize = {19, 18};
    inline int playerCountWidth = 0;
    inline int distanceWidth = 0;
}

namespace Style {
    inline Color primaryColor = WHITE;
    inline Color hoverColor = GRAY;
    inline Color secondaryColor = BLACK;
    inline Color greenColor = GREEN;
    inline Color bgColor1 = {0, 32, 0, 255};
    inline Color bgColor2 = {0, 64, 0, 255};
}

enum Settings {
    ShowFlags = 0,
    UseFancyCursor = 1,
    ScrollingBG = 2,
};

enum Screens {
    RayballLogo = 0,
    ServerList = 1,
    Configuration = 2,
};

enum Buttons {
    LastPage = 0,
    NextPage = 1,
    Refresh = 2,
};
