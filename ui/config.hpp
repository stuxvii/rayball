#include "raylib.h"

bool ShowFlagImages = false;
bool FancyCursor = true;
int FPS = 240;
int fontSize = 13;
Texture2D flagTextureAtlas;

enum Settings {
    UnlockFPS = 0,
    ShowFlags = 1,
    UseFancyCursor = 2,
};