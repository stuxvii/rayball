#include "config.hpp"
#include "flag_coords.hpp"
#include "../extlib/harvesine/harvesine.cpp"
#include <raylib.h>
#include <string>
#include <format>

struct Button {
  std::string txt;

  //virtual ~Button() = default;
  Button(std::string t) : txt(std::move(t)) {}
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
    Color bg = mouse_over ? Style::hoverColor : Style::secondaryColor;
    DrawRectangleRec(rect, bg);
    DrawText(txt.c_str(), rect.x + Layout::spacing, rect.y + Layout::spacing, Layout::fontSize, Style::primaryColor);
  }
  virtual void RenderFont(Rectangle rect, bool mouse_over, Font font) {
    Color bg = mouse_over ? Style::hoverColor : Style::secondaryColor;
    DrawRectangleRec(rect, bg);
    DrawTextEx(font, txt.c_str(),
               Vector2{
                   rect.x,
                   rect.y,
               },
               Layout::fontSize, 0, Style::primaryColor);
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
  float distance_km;
  bool locked;

  Room(std::string t, std::string i, std::string c, Vector2 d, std::string p, std::string max_p, Rectangle r, std::string pl, bool l) :
      Button(std::move(t)),
      id(std::move(i)),
      country(std::move(c)),
      coords(std::move(d)),
      players(std::move(p)),
      max_players(std::move(max_p)),
      map_rec(std::move(r)),
      player_label(std::move(pl)),
      locked(std::move(l))
  {
    distance_km = calculate_distance(Config::Latitude, Config::Longitude, coords.x, coords.y);
  }

  bool Draw(Rectangle rect) override {
    Rectangle flagBGRect = {
      rect.x + rect.width + Layout::spacing*2 + Layout::playerCountWidth,
      rect.y,
      Layout::flagSize.y+Layout::distanceWidth,
      rect.height
    };

    int pcl_offset = rect.width + rect.x + Layout::spacing;
    Rectangle player_count_rec = {(float)pcl_offset, rect.y, (float)Layout::playerCountWidth, rect.height};

    DrawRectangleRec(player_count_rec, Style::secondaryColor);
    DrawText(player_label.c_str(),
             player_count_rec.x + Layout::spacing,
             rect.y + Layout::spacing, Layout::fontSize, Style::primaryColor);


    DrawRectangleRec(flagBGRect, Style::secondaryColor);

    if (Config::ShowFlagImages) {
      DrawTextureRec(Layout::flagTextureAtlas, map_rec, (Vector2){flagBGRect.x + Layout::spacing, rect.y + Layout::spacing}, WHITE);
    } else {
      DrawText(country.c_str(), flagBGRect.x + Layout::spacing, rect.y + Layout::spacing, Layout::fontSize, Style::primaryColor);
    }
    DrawText(std::format("{}km", round(distance_km)).c_str(), Layout::flagSize.x + flagBGRect.x, rect.y + Layout::spacing, Layout::fontSize, Style::primaryColor);

    return Button::Draw(rect);
  }
};

struct ToggleButton : public Button {
  bool isToggled = false;

  ToggleButton(const std::string t, bool toggled = false) : Button(std::move(t)) {
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
    Color activeHoverBG = (mouse_over ? Style::hoverColor : Style::greenColor);
    Color bg = isToggled ? activeHoverBG : (mouse_over ? Style::hoverColor : Style::secondaryColor);
    Color fg = isToggled ? Style::secondaryColor : Style::primaryColor;
    DrawRectangleRec(rect, bg);
    DrawText(txt.c_str(), rect.x + Layout::spacing, rect.y + Layout::spacing, Layout::fontSize, fg);
  }
};
