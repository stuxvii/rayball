#include "config.hpp"
#include "flag_coords.hpp"
#include <raylib.h>
#include <string>

struct Button {
  std::string txt;

  virtual ~Button() = default;

public:
  virtual bool Draw(Rectangle rect) {
    Vector2 m = GetMousePosition();
    bool mouse_over = CheckCollisionPointRec(m, rect);
    bool clicked = IsMouseButtonPressed(MOUSE_BUTTON_LEFT);

    Render(rect, mouse_over);
    return (mouse_over && clicked);
  }

  virtual bool DrawFont(Rectangle rect, Font font) {
    Vector2 m = GetMousePosition();
    bool mouse_over = CheckCollisionPointRec(m, rect);
    bool clicked = IsMouseButtonPressed(MOUSE_BUTTON_LEFT);

    RenderFont(rect, mouse_over, font);
    return (mouse_over && clicked);
  }

protected:
  virtual void Render(Rectangle rect, bool mouse_over) {
    Color bg = mouse_over ? WHITE : LIGHTGRAY;
    DrawRectangleRec(rect, bg);
    DrawText(txt.c_str(), rect.x + 5, rect.y + 2, fontSize, BLACK);
  }
  virtual void RenderFont(Rectangle rect, bool mouse_over, Font font) {
    Color bg = mouse_over ? WHITE : LIGHTGRAY;
    DrawRectangleRec(rect, bg);
    DrawTextEx(font, txt.c_str(),
               Vector2{
                   rect.x,
                   rect.y,
               },
               fontSize, 0, BLACK);
  }
};

struct Room : public Button {
  std::string id;
  std::string country;
  Vector2 coords;
  std::string players;
  std::string max_players;
  Rectangle map_rec;
  std::string player_label;

  Room(std::string t, std::string i, std::string c, Vector2 d, std::string p, std::string max_p, Rectangle r, std::string pl)
      : 
      id(std::move(i)),
      country(std::move(c)),
      coords(std::move(d)),
      players(std::move(p)),
      max_players(std::move(max_p)),
      map_rec(std::move(r)),
      player_label(std::move(pl))
  {
    this->txt = std::move(t);
  }

  bool Draw(Rectangle rect) override {
    DrawRectangle(rect.x - 19, rect.y, 18, rect.height, LIGHTGRAY);

    if (ShowFlagImages) {
      DrawTextureRec(flagTextureAtlas, map_rec, (Vector2){rect.x - 18, rect.y + 1}, WHITE);
    } else {
      DrawText(country.c_str(), rect.x - 18, rect.y + 1, fontSize, BLACK);
    }

    int pclt_size = MeasureText(player_label.c_str(), fontSize);
    int pcl_size = GetScreenWidth() - rect.width - rect.x - 4;
    int pcl_x_pos = /*rect.x + rect.width + */(GetScreenWidth() - pcl_size - 2);

    Rectangle player_count_rec = {(float)pcl_x_pos, rect.y, (float)pcl_size, rect.height};

    DrawRectangleRec(player_count_rec, LIGHTGRAY);
    DrawText(player_label.c_str(),
             player_count_rec.x + player_count_rec.width - pclt_size - 2,
             rect.y + 1, fontSize, BLACK);
    return Button::Draw(rect);
  }
};

struct ToggleButton : public Button {
  bool isToggled = false;

  ToggleButton(const std::string t, bool toggled = false) {
    txt = t;
    isToggled = toggled;
  }

  bool Draw(Rectangle rect) override {
    if (Button::Draw(rect)) {
      isToggled = !isToggled;
    }
    return isToggled;
  }

  void Render(Rectangle rect, bool mouse_over) override {
    Color bg = isToggled ? GREEN : (mouse_over ? WHITE : LIGHTGRAY);
    DrawRectangleRec(rect, bg);
    DrawText(txt.c_str(), rect.x + 5, rect.y + 5, 14, BLACK);
  }
};
