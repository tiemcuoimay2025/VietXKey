#include "theme.h"
#include <windows.h>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include <algorithm>

namespace vietflow {

static const void* s_embedded_font_data = nullptr;
static size_t s_embedded_font_size = 0;
static const void* s_heading_font_data = nullptr;
static size_t s_heading_font_size = 0;
static ImFont* s_heading_font = nullptr;
static const void* s_tray_font_data = nullptr;
static size_t s_tray_font_size = 0;
static ImFont* s_tray_font = nullptr;
static float s_ui_scale = 1.0f;
static bool s_dark_theme = true;
void SetDarkTheme(bool dark) { s_dark_theme = dark; }
bool IsDarkTheme() { return s_dark_theme; }

void SetEmbeddedFontData(const void* data, size_t size) {
    s_embedded_font_data = data;
    s_embedded_font_size = size;
}

void SetHeadingFontData(const void* data, size_t size) {
    s_heading_font_data = data;
    s_heading_font_size = size;
}

ImFont* HeadingFont() {
    return s_heading_font ? s_heading_font : ImGui::GetFont();
}

static void AddHeadingFont(ImGuiIO& io, float dpi_scale, const ImWchar* ranges) {
    s_heading_font = nullptr;
    if (!s_heading_font_data || s_heading_font_size == 0) return;
    void* copy = malloc(s_heading_font_size);
    if (!copy) return;
    memcpy(copy, s_heading_font_data, s_heading_font_size);
    ImFontConfig cfg;
    cfg.FontDataOwnedByAtlas = true;
    s_heading_font = io.Fonts->AddFontFromMemoryTTF(copy, (int)s_heading_font_size, 22.5f * dpi_scale, &cfg, ranges);
}

void SetTrayFontData(const void* data, size_t size) {
    s_tray_font_data = data;
    s_tray_font_size = size;
}

ImFont* TrayFont() {
    return s_tray_font;
}

static void AddTrayFont(ImGuiIO& io, float dpi_scale) {
    s_tray_font = nullptr;
    if (!s_tray_font_data || s_tray_font_size == 0) return;

    void* font_copy = malloc(s_tray_font_size);
    if (!font_copy) return;
    memcpy(font_copy, s_tray_font_data, s_tray_font_size);

    ImFontConfig cfg;
    cfg.FontDataOwnedByAtlas = true;
    cfg.OversampleH = 1;
    cfg.OversampleV = 1;
    cfg.PixelSnapH = true;
    static const ImWchar tray_ranges[] = { 0x45, 0x45, 0x56, 0x56, 0 };
    s_tray_font = io.Fonts->AddFontFromMemoryTTF(
        font_copy, (int)s_tray_font_size, 42.0f * dpi_scale, &cfg, tray_ranges);
}

float UiScale() {
    return s_ui_scale > 0.0f ? s_ui_scale : 1.0f;
}

void SetupFonts(ImGuiIO& io, float dpi_scale) {
    if (dpi_scale <= 0.0f) dpi_scale = 1.0f;
    float font_size = 20.0f * dpi_scale;

    ImFontConfig font_cfg;
    font_cfg.OversampleH = 2;
    font_cfg.OversampleV = 1;
    font_cfg.PixelSnapH = true;

    // Only the glyphs the UI actually renders. Loading the whole Nerd Font PUA
    // range (0xE000-0xF8FF) inflates the atlas by thousands of unused icons and
    // costs tens of MB of VRAM at high DPI.
    static const ImWchar icon_ranges[] = {
        0x2022, 0x2022, // bullet used by list rows
        0xF00C, 0xF00C, // NF_CHECK
        0xF013, 0xF013, // NF_COG
        0xF021, 0xF021, // NF_CONVERT
        0xF05A, 0xF05A, // NF_INFO
        0xF067, 0xF067, // NF_PLUS
        0xF07B, 0xF07B, // NF_FOLDER
        0xF0C5, 0xF0C5, // NF_COPY
        0xF0C7, 0xF0C7, // NF_SAVE
        0xF0E7, 0xF0E7, // NF_BOLT
        0xF0EC, 0xF0EC, // NF_EXCHANGE
        0xF11B, 0xF11B, // NF_GAMEPAD
        0xF11C, 0xF11C, // NF_KEYBOARD
        0xF1F8, 0xF1F8, // NF_TRASH
        0xF1FC, 0xF1FC, // NF_PAINT
        0
    };

    ImFontGlyphRangesBuilder builder;
    builder.AddRanges(io.Fonts->GetGlyphRangesDefault());
    builder.AddRanges(io.Fonts->GetGlyphRangesVietnamese());
    builder.AddRanges(icon_ranges);

    static ImVector<ImWchar> full_ranges;
    builder.BuildRanges(&full_ranges);

    // 1. Direct in-memory font embedded in Rust binary (zero disk I/O, zero extraction)
    if (s_embedded_font_data && s_embedded_font_size > 0) {
        void* font_copy = malloc(s_embedded_font_size);
        if (font_copy) {
            memcpy(font_copy, s_embedded_font_data, s_embedded_font_size);
            font_cfg.FontDataOwnedByAtlas = true;
            io.Fonts->AddFontFromMemoryTTF(font_copy, (int)s_embedded_font_size, font_size, &font_cfg, full_ranges.Data);
            AddHeadingFont(io, dpi_scale, full_ranges.Data);
            AddTrayFont(io, dpi_scale);
            return;
        }
    }

    // 2. Fallback: Windows system font
    const char* fallback_font = "C:\\Windows\\Fonts\\segoeui.ttf";
    if (GetFileAttributesA(fallback_font) != INVALID_FILE_ATTRIBUTES) {
        io.Fonts->AddFontFromFileTTF(fallback_font, font_size, &font_cfg, full_ranges.Data);
    } else {
        io.Fonts->AddFontDefault(&font_cfg);
    }
    AddHeadingFont(io, dpi_scale, full_ranges.Data);
    AddTrayFont(io, dpi_scale);
}

void ApplyTheme(float dpi_scale) {
    if (dpi_scale <= 0.0f) dpi_scale = 1.0f;
    s_ui_scale = dpi_scale;
    ImGuiStyle& style = ImGui::GetStyle();

    // Start from unscaled metrics so theme/DPI switches never compound sizes.
    style = ImGuiStyle();
    if (IsDarkTheme()) ImGui::StyleColorsDark(&style);
    else ImGui::StyleColorsLight(&style);

    style.WindowPadding     = ImVec2(20.0f, 20.0f);
    style.FramePadding      = ImVec2(12.0f, 7.0f);
    style.CellPadding       = ImVec2(10.0f, 8.0f);
    style.ItemSpacing       = ImVec2(10.0f, 12.0f);
    style.ItemInnerSpacing  = ImVec2(6.0f, 4.0f);
    style.IndentSpacing     = 18.0f;
    style.ScrollbarSize     = 8.0f;
    style.GrabMinSize       = 10.0f;

    style.WindowBorderSize  = 1.0f;
    style.ChildBorderSize   = 0.0f;
    style.PopupBorderSize   = 1.0f;
    style.FrameBorderSize   = 0.0f;

    style.WindowRounding    = 12.0f;
    style.ChildRounding     = 16.0f;
    style.FrameRounding     = 8.0f;
    style.PopupRounding     = 12.0f;
    style.ScrollbarRounding = 8.0f;
    style.GrabRounding      = 8.0f;
    style.TabRounding       = 8.0f;
    style.TabBorderSize     = 0.0f;

    // SeparatorText clean style
    style.SeparatorTextBorderSize = 1.0f;
    style.SeparatorTextAlign = ImVec2(0.0f, 0.5f);
    style.SeparatorTextPadding = ImVec2(6.0f, 4.0f);

    style.ScaleAllSizes(dpi_scale);

    ImVec4* colors = style.Colors;
    colors[ImGuiCol_ScrollbarBg] = ImVec4(0.08f, 0.08f, 0.09f, 0.70f);
    colors[ImGuiCol_Text]               = ImVec4(0.92f, 0.94f, 0.89f, 1.00f);
    colors[ImGuiCol_TextDisabled]       = ImVec4(0.59f, 0.63f, 0.56f, 1.00f);
    colors[ImGuiCol_WindowBg]           = ImVec4(0.075f, 0.075f, 0.080f, 1.00f);
    colors[ImGuiCol_ChildBg]            = ImVec4(0.12f, 0.12f, 0.13f, 0.00f);
    colors[ImGuiCol_PopupBg]            = ImVec4(0.13f, 0.13f, 0.14f, 0.99f);
    colors[ImGuiCol_Border]             = ImVec4(0.30f, 0.30f, 0.32f, 0.65f);
    colors[ImGuiCol_FrameBg]            = ImVec4(0.17f, 0.17f, 0.18f, 1.00f);
    colors[ImGuiCol_FrameBgHovered]     = ImVec4(0.23f, 0.23f, 0.25f, 1.00f);
    colors[ImGuiCol_FrameBgActive]      = ImVec4(0.29f, 0.29f, 0.31f, 1.00f);

    colors[ImGuiCol_Button]             = ImVec4(0.31f, 0.40f, 0.23f, 1.00f);
    colors[ImGuiCol_ButtonHovered]      = ImVec4(0.39f, 0.49f, 0.29f, 1.00f);
    colors[ImGuiCol_ButtonActive]       = ImVec4(0.25f, 0.33f, 0.19f, 1.00f);

    colors[ImGuiCol_Header]             = ImVec4(0.29f, 0.38f, 0.22f, 0.80f);
    colors[ImGuiCol_HeaderHovered]      = ImVec4(0.34f, 0.44f, 0.26f, 1.00f);
    colors[ImGuiCol_HeaderActive]       = ImVec4(0.40f, 0.50f, 0.28f, 1.00f);

    colors[ImGuiCol_CheckMark]          = ImVec4(0.73f, 0.82f, 0.52f, 1.00f);
    colors[ImGuiCol_CheckboxSelectedBg] = ImVec4(0.23f, 0.29f, 0.17f, 1.00f);
    colors[ImGuiCol_SliderGrab]         = ImVec4(0.55f, 0.67f, 0.36f, 1.00f);
    colors[ImGuiCol_SliderGrabActive]   = ImVec4(0.73f, 0.82f, 0.52f, 1.00f);

    colors[ImGuiCol_Separator]          = ImVec4(0.26f, 0.26f, 0.28f, 1.00f);
    colors[ImGuiCol_SeparatorHovered]   = ImVec4(0.43f, 0.54f, 0.31f, 1.00f);
    colors[ImGuiCol_SeparatorActive]    = ImVec4(0.73f, 0.82f, 0.52f, 1.00f);

    colors[ImGuiCol_Tab]                = ImVec4(0.16f, 0.16f, 0.18f, 0.70f);
    colors[ImGuiCol_TabHovered]         = ImVec4(0.31f, 0.41f, 0.23f, 0.90f);
    colors[ImGuiCol_TabSelected]        = ImVec4(0.29f, 0.39f, 0.21f, 1.00f);
    colors[ImGuiCol_TabDimmed]          = ImVec4(0.12f, 0.12f, 0.13f, 0.70f);
    colors[ImGuiCol_TabDimmedSelected]  = ImVec4(0.22f, 0.30f, 0.16f, 1.00f);
    colors[ImGuiCol_TabSelectedOverline]       = ImVec4(0.0f, 0.0f, 0.0f, 0.0f);
    colors[ImGuiCol_TabDimmedSelectedOverline] = ImVec4(0.0f, 0.0f, 0.0f, 0.0f);
    colors[ImGuiCol_NavCursor]          = ImVec4(0.73f, 0.82f, 0.52f, 1.00f);
    colors[ImGuiCol_TextSelectedBg]     = ImVec4(0.40f, 0.53f, 0.29f, 0.65f);
    colors[ImGuiCol_TextLink]           = ImVec4(0.73f, 0.82f, 0.52f, 1.00f);
    colors[ImGuiCol_TitleBg]            = ImVec4(0.12f, 0.12f, 0.13f, 1.00f);
    colors[ImGuiCol_TitleBgActive]      = ImVec4(0.20f, 0.20f, 0.22f, 1.00f);
    colors[ImGuiCol_ScrollbarGrab]      = ImVec4(0.32f, 0.32f, 0.34f, 1.00f);
    colors[ImGuiCol_ScrollbarGrabHovered] = ImVec4(0.44f, 0.44f, 0.46f, 1.00f);
    colors[ImGuiCol_ScrollbarGrabActive] = ImVec4(0.56f, 0.56f, 0.58f, 1.00f);
    colors[ImGuiCol_ResizeGrip]         = ImVec4(0.38f, 0.48f, 0.25f, 0.25f);
    colors[ImGuiCol_ResizeGripHovered]  = ImVec4(0.51f, 0.61f, 0.36f, 0.70f);
    colors[ImGuiCol_ResizeGripActive]   = ImVec4(0.73f, 0.82f, 0.52f, 1.00f);
    colors[ImGuiCol_TableHeaderBg]      = ImVec4(0.17f, 0.17f, 0.18f, 1.00f);
    colors[ImGuiCol_TableRowBgAlt]      = ImVec4(0.32f, 0.32f, 0.34f, 0.18f);
    colors[ImGuiCol_WindowBg] = ImVec4(0,0,0,0);
    colors[ImGuiCol_ModalWindowDimBg] = ImVec4(0.02f,0.03f,0.02f,0.55f);
    if (!IsDarkTheme()) {
        auto set = [&](ImGuiCol key, int r, int g, int b, float a = 1.0f) {
            colors[key] = ImVec4(r/255.0f,g/255.0f,b/255.0f,a);
        };
        set(ImGuiCol_Text,34,42,31); set(ImGuiCol_TextDisabled,99,111,88);
        set(ImGuiCol_ChildBg,255,255,255,0); set(ImGuiCol_PopupBg,252,253,250);
        set(ImGuiCol_Border,173,184,161,.6f);
        set(ImGuiCol_FrameBg,231,236,224); set(ImGuiCol_FrameBgHovered,220,230,209);
        set(ImGuiCol_FrameBgActive,208,222,190);
        set(ImGuiCol_Button,209,222,190); set(ImGuiCol_ButtonHovered,190,210,163);
        set(ImGuiCol_ButtonActive,174,196,144);
        set(ImGuiCol_Header,220,231,205); set(ImGuiCol_HeaderHovered,207,223,186);
        set(ImGuiCol_HeaderActive,191,212,164);
        set(ImGuiCol_CheckMark,66,96,38); set(ImGuiCol_CheckboxSelectedBg,212,228,192);
        set(ImGuiCol_SliderGrab,105,136,67); set(ImGuiCol_SliderGrabActive,76,109,42);
        set(ImGuiCol_Separator,187,198,177,.7f); set(ImGuiCol_SeparatorHovered,112,143,76);
        set(ImGuiCol_SeparatorActive,76,109,42);
        set(ImGuiCol_Tab,236,240,230); set(ImGuiCol_TabHovered,207,223,186);
        set(ImGuiCol_TabSelected,218,232,201); set(ImGuiCol_TabDimmed,237,241,232);
        set(ImGuiCol_TabDimmedSelected,225,234,213);
        set(ImGuiCol_NavCursor,76,109,42); set(ImGuiCol_TextSelectedBg,165,193,132,.55f);
        set(ImGuiCol_TextLink,65,100,34); set(ImGuiCol_TitleBg,235,240,228);
        set(ImGuiCol_TitleBgActive,224,234,213); set(ImGuiCol_ScrollbarBg,229,234,222,.5f);
        set(ImGuiCol_ScrollbarGrab,174,188,157); set(ImGuiCol_ScrollbarGrabHovered,143,165,117);
        set(ImGuiCol_ScrollbarGrabActive,109,140,75);
        set(ImGuiCol_TableHeaderBg,228,235,219); set(ImGuiCol_TableRowBg,255,255,255,0);
        set(ImGuiCol_TableRowBgAlt,136,158,112,.07f);
        set(ImGuiCol_TableBorderStrong,177,191,162,.65f); set(ImGuiCol_TableBorderLight,193,205,180,.45f);
        set(ImGuiCol_ModalWindowDimBg,18,25,12,.27f);
    }
}

void SectionHeader(const char* title) {
    ImGui::Spacing();
    ImGui::PushFont(HeadingFont(), TextPx(18.0f));
    ImGui::SeparatorText(title);
    ImGui::PopFont();
}

void HotkeyBadge(const char* text) {
    ImVec2 p = ImGui::GetCursorScreenPos();
    ImVec2 size = ImGui::CalcTextSize(text);
    size.x += Px(16.0f);
    size.y += Px(8.0f);

    ImDrawList* draw_list = ImGui::GetWindowDrawList();
    ImU32 bg_col = ThemeColor(IM_COL32(40, 46, 43, 255), IM_COL32(234, 240, 225, 255));
    ImU32 border_col = ThemeColor(IM_COL32(83, 95, 76, 255), IM_COL32(167, 185, 146, 255));

    draw_list->AddRectFilled(p, ImVec2(p.x + size.x, p.y + size.y), bg_col, Px(7.0f));
    draw_list->AddRect(p, ImVec2(p.x + size.x, p.y + size.y), border_col, Px(7.0f));

    ImVec2 text_pos(p.x + Px(8.0f), p.y + Px(4.0f));
    draw_list->AddText(text_pos, ThemeTextColor(IM_COL32(218, 231, 191, 255)), text);

    ImGui::Dummy(size);
}

bool NavItem(const char* label, bool selected) {
    const ImVec2 p = ImGui::GetCursorScreenPos();
    const ImVec2 size(ImGui::GetContentRegionAvail().x, Px(44.0f));
    ImDrawList* draw = ImGui::GetWindowDrawList();
    if (selected) {
        GradientPanel(draw, p, ImVec2(p.x + size.x, p.y + size.y),
                      IM_COL32(63, 84, 43, 255), IM_COL32(88, 104, 53, 255), Px(10.0f));
    }
    ImGui::PushStyleColor(ImGuiCol_Button, ImVec4(0, 0, 0, 0));
    ImGui::PushStyleColor(ImGuiCol_ButtonHovered, ImVec4(0.5f, 0.6f, 0.4f, 0.12f));
    ImGui::PushStyleColor(ImGuiCol_ButtonActive, ImVec4(0.5f, 0.6f, 0.4f, 0.20f));
    ImGui::PushID(label);
    const bool clicked = ImGui::Button("##nav", size);
    ImGui::PopID();
    ImGui::PopStyleColor(3);
    draw->AddText(ImVec2(p.x + Px(14.0f), p.y + (size.y - ImGui::GetFontSize()) * 0.5f),
                  selected ? IM_COL32(244, 248, 235, 255) : ThemeTextColor(IM_COL32(172, 184, 160, 255)), label);
    return clicked;
}

void GradientPanel(ImDrawList* draw, ImVec2 min, ImVec2 max, ImU32 left, ImU32 right, float rounding) {
    const int first = draw->VtxBuffer.Size;
    draw->AddRectFilled(min, max, IM_COL32_WHITE, rounding);
    const ImVec4 a = ImGui::ColorConvertU32ToFloat4(left);
    const ImVec4 b = ImGui::ColorConvertU32ToFloat4(right);
    for (int i = first; i < draw->VtxBuffer.Size; ++i) {
        ImDrawVert& v = draw->VtxBuffer[i];
        const float t = (v.pos.x - min.x) / (max.x - min.x);
        const ImU32 alpha = v.col & IM_COL32_A_MASK;
        v.col = (ImGui::ColorConvertFloat4ToU32(ImVec4(a.x + (b.x-a.x)*t, a.y + (b.y-a.y)*t,
                                                     a.z + (b.z-a.z)*t, 1.0f)) & ~IM_COL32_A_MASK) | alpha;
    }
}

bool ToggleSwitch(const char* id, bool* value) {
    const ImVec2 p = ImGui::GetCursorScreenPos();
    const ImVec2 size(Px(44.0f), Px(25.0f));
    const bool clicked = ImGui::InvisibleButton(id, size, ImGuiButtonFlags_EnableNav);
    if (clicked) *value = !*value;
    ImDrawList* draw = ImGui::GetWindowDrawList();
    const bool hovered = ImGui::IsItemHovered();
    const ImU32 track = *value ? (hovered ? IM_COL32(140, 159, 89, 255) : IM_COL32(113, 138, 68, 255))
                               : (hovered ? ThemeColor(IM_COL32(80,91,68,255),IM_COL32(185,197,173,255))
                                          : ThemeColor(IM_COL32(58,68,49,255),IM_COL32(205,214,194,255)));
    draw->AddRectFilled(p, ImVec2(p.x + size.x, p.y + size.y), ImGui::GetColorU32(track), size.y * 0.5f);
    const float cx = *value ? p.x + size.x - size.y * 0.5f : p.x + size.y * 0.5f;
    draw->AddCircleFilled(ImVec2(cx, p.y + size.y * 0.5f), Px(9.0f), ImGui::GetColorU32(IM_COL32(240, 244, 228, 255)));
    return clicked;
}

ImU32 ThemeTextColor(ImU32 color) {
    if (IsDarkTheme()) return color;
    const int a = (color >> IM_COL32_A_SHIFT) & 255;
    const int r = (color >> IM_COL32_R_SHIFT) & 255;
    const int g = (color >> IM_COL32_G_SHIFT) & 255;
    const int b = (color >> IM_COL32_B_SHIFT) & 255;
    if (r > g * 1.2f && r > b * 1.2f) return IM_COL32(153,54,45,a);
    const float l = .2126f*r + .7152f*g + .0722f*b;
    if (l > 210) return IM_COL32(33,43,28,a);
    if (l > 160) return IM_COL32(67,91,45,a);
    return IM_COL32(98,113,85,a);
}

void DrawAppBackground(ImDrawList* draw, ImVec2 min, ImVec2 max) {
    const ImVec4 outer = ImGui::ColorConvertU32ToFloat4(ThemeColor(IM_COL32(19,23,28,255),IM_COL32(230,235,225,255)));
    const ImVec4 inner = ImGui::ColorConvertU32ToFloat4(ThemeColor(IM_COL32(40,47,55,255),IM_COL32(255,255,253,255)));
    auto color = [&](float u,float v) {
        const float x=(u-.50f)/.85f, y=(v-.13f)/1.0f;
        float t=(std::max)(0.0f,1.0f-std::sqrt(x*x+y*y));
        t=t*t*(3.0f-2.0f*t);
        return ImGui::ColorConvertFloat4ToU32(ImVec4(outer.x+(inner.x-outer.x)*t,
            outer.y+(inner.y-outer.y)*t,outer.z+(inner.z-outer.z)*t,1));
    };
    constexpr int nx=28, ny=22;
    for(int y=0;y<ny;++y) for(int x=0;x<nx;++x) {
        float u0=x/(float)nx,u1=(x+1)/(float)nx,v0=y/(float)ny,v1=(y+1)/(float)ny;
        draw->AddRectFilledMultiColor(ImVec2(min.x+(max.x-min.x)*u0,min.y+(max.y-min.y)*v0),
            ImVec2(min.x+(max.x-min.x)*u1,min.y+(max.y-min.y)*v1),
            color(u0,v0),color(u1,v0),color(u1,v1),color(u0,v1));
    }
}

void DrawBrandKeycap(ImDrawList* draw, ImVec2 p, float s) {
    draw->AddRectFilled(ImVec2(p.x+s*.015f,p.y+s*.08f),ImVec2(p.x+s*.985f,p.y+s),IM_COL32(19,30,11,80),s*.21f);
    GradientPanel(draw, p, ImVec2(p.x+s,p.y+s*.94f), IM_COL32(69,90,43,255),IM_COL32(48,66,30,255),s*.20f);
    const ImVec2 face(p.x+s*.055f,p.y+s*.018f), end(p.x+s*.945f,p.y+s*.82f);
    GradientPanel(draw,face,end,IM_COL32(186,208,143,255),IM_COL32(121,155,77,255),s*.17f);
    draw->AddRect(face,end,IM_COL32(235,246,205,110),s*.17f,0,(std::max)(1.0f,s*.012f));
    const ImVec2 a(p.x+s*.30f,p.y+s*.32f), b(p.x+s*.50f,p.y+s*.66f), c(p.x+s*.70f,p.y+s*.32f);
    draw->AddLine(ImVec2(a.x,a.y+s*.015f),ImVec2(b.x,b.y+s*.015f),IM_COL32(39,66,18,70),s*.10f);
    draw->AddLine(ImVec2(b.x,b.y+s*.015f),ImVec2(c.x,c.y+s*.015f),IM_COL32(39,66,18,70),s*.10f);
    draw->AddLine(a,b,IM_COL32(247,251,235,255),s*.09f);
    draw->AddLine(b,c,IM_COL32(247,251,235,255),s*.09f);
    draw->AddLine(ImVec2(p.x+s*.35f,p.y+s*.89f),ImVec2(p.x+s*.65f,p.y+s*.89f),IM_COL32(154,183,113,160),s*.025f);
}

} // namespace vietflow
