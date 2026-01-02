#include "raylib.h"

const int MAX_TRAIL_LENGTH = 3;
Vector2 trailPositions[MAX_TRAIL_LENGTH] = {};
const float TRAIL_DECAY = 0.025f;
const float CURSOR_SIZE = 10.0f;
float trailTimer = 0.0f;

void DrawCursorTrail(float dt) {
    Vector2 mousePosition = GetMousePosition();
    trailTimer += dt;
    if (trailTimer > TRAIL_DECAY) {
        for (int i=MAX_TRAIL_LENGTH-1; i > 0; i--) {
            trailPositions[i] = trailPositions[i-1];
        }

        trailPositions[0] = mousePosition;

        trailTimer = 0.0f;
    }
    for (int i = 0; i <MAX_TRAIL_LENGTH; i++) {
        if ((trailPositions[i].x != 0.0f) || (trailPositions[i].y != 0.0f)) {
            float ratio = (float)(MAX_TRAIL_LENGTH - i)/MAX_TRAIL_LENGTH;
            Color trailColor = Fade(WHITE, ratio*0.5f);
            float trailRadius = CURSOR_SIZE*ratio;
            DrawCircleV(trailPositions[i], trailRadius, trailColor);
        }
    }

    DrawCircleV(mousePosition, CURSOR_SIZE, WHITE);
}