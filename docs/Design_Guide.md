# Project: IRON-VOX // UX Specification v4.0 (Rust/egui Edition)

**Tech Stack:** Rust, eframe, egui.
**Design System:** Industrial Minimalist.
**Aesthetic Core:** "Tekkadan Hardware" (Dark) vs "Mobile Suit Frame" (Light).
**Goal:** A high-functionality Karaoke app that feels rugged and mechanical (IBO-inspired) but retains standard, familiar music player usability.

## 1. Visual Philosophy: "Form Follows Function"

Instead of a raw military simulation, imagine this is a **civilian music player built with military-grade parts**.

* **Legibility First:** Decorative elements are background texture only. Text must pop.
* **Familiar Layouts:** We adhere to standard mental models.
* **The "Chamfer" Rule:** To maintain the IBO feel, use custom **painters** to draw containers with cut corners (45-degree chamfer) instead of rounded rectangles.

## 2. Color Systems (Faction Coded)

We use distinct palettes representing the Pilot's Faction (Dark) and the Machine (Light).

### Mode A: "Iron Flower" (Dark / Tekkadan Theme)

*Inspired by the Tekkadan green military jackets, Mars dust, and the red logo. Earthy, gritty, high contrast.*

| Role | Hex | egui::Color32 | Usage |
| :--- | :--- | :--- | :--- |
| **App Background** | `#111311` | `Color32::from_rgb(17, 19, 17)` | **Void Green**. Very dark, almost black green. |
| **Card Surface** | `#222924` | `Color32::from_rgb(34, 41, 36)` | **Uniform Green**. Resembles the jacket fabric. |
| **Primary Action** | `#A82028` | `Color32::from_rgb(168, 32, 40)` | **Flower Red**. The iconic Tekkadan logo red. High visibility. |
| **Secondary/UI** | `#3A403C` | `Color32::from_rgb(58, 64, 60)` | **Gunmetal**. Borders, dividers, inactive buttons. |
| **Accent/Alert** | `#D48D3B` | `Color32::from_rgb(212, 141, 59)` | **Mars Dust**. Used for focus states and warnings. |
| **Text Primary** | `#E8E6E3` | `Color32::from_rgb(232, 230, 227)` | **Bone White**. Soft white for max readability against dark green. |
| **Text Muted** | `#959B96` | `Color32::from_rgb(149, 155, 150)` | **Faded Canvas**. Metadata text. |

### Mode B: "White Devil" (Light / Barbatos Theme)

*Inspired by the ASW-G-08 Gundam Barbatos. Ceramic white armor, deep blue reactor, yellow vents, red chin.*

| Role | Hex | egui::Color32 | Usage |
| :--- | :--- | :--- | :--- |
| **App Background** | `#F0F2F5` | `Color32::from_rgb(240, 242, 245)` | **Hangar Wall**. Cool off-white. |
| **Card Surface** | `#FFFFFF` | `Color32::WHITE` | **Ceramic Armor**. Pure white panels with sharp shadows. |
| **Primary Action** | `#18458B` | `Color32::from_rgb(24, 69, 139)` | **Cobalt Blue**. The chest armor color. Used for Play/CTA. |
| **Secondary/UI** | `#E5E7EB` | `Color32::from_rgb(229, 231, 235)` | **Inner Frame**. Light grey for borders/inactive. |
| **Accent** | `#EBC934` | `Color32::from_rgb(235, 201, 52)` | **V-Fin Yellow**. Used for sliders/vents. *Note: Use dark text on this.* |
| **Alert** | `#C91A25` | `Color32::from_rgb(201, 26, 37)` | **Chin Red**. Destructive actions (Delete/Remove). |
| **Text Primary** | `#1F2937` | `Color32::from_rgb(31, 41, 55)` | **Oil Black**. High contrast against white. |

## 3. Typography

* **Headings:** [**Chakra Petch**](https://fonts.google.com/specimen/Chakra+Petch) (Bold).
* **Lyrics:** [**Rajdhani**](https://fonts.google.com/specimen/Rajdhani) (Bold).
* **UI/Lists:** [**Roboto**](https://fonts.google.com/specimen/Roboto).

## 4. UI Components

### A. The "Armor Card" (Chamfered)

Use `ui.painter()` to draw polygons.

* **Dark Mode:** Solid fill `#222924`, Left Border 2px `#A82028`.
* **Light Mode:** Solid fill `#FFFFFF`, Drop Shadow, Left Border 2px `#18458B`.

### B. Controls

* **Buttons:** Rectangles with 45deg cut corners.
* **Sliders:** "Piston" style.
  * *Tekkadan:* Track `#3A403C`, Fill `#D48D3B` (Orange).
  * *Barbatos:* Track `#E5E7EB`, Fill `#EBC934` (Yellow).

## 5. Layout Blueprint: Settings Panel

This panel opens via the "Settings" tab or a gear icon. It mimics a **Mobile Suit Maintenance Console**.

**Visual Metaphor:**

* **Sections** are distinct "Systems" (Audio, Video, Network).
* **Toggles** look like heavy mechanical switches.
* **Inputs** look like terminal command lines.

```text
+-----------------------------------------------------------------------+
|  TopPanel: [ < Back ]   SYSTEM CONFIGURATION (SETTINGS)               |
+--------------------------+--------------------------------------------+
|  SidePanel (Categories)  |  CentralPanel (Viewport)                   |
|  [Background: Darker]    |  [Background: Card Surface]                |
|                          |                                            |
|  [ > ] AUDIO SYSTEM      |  +--------------------------------------+  |
|  [   ] DISPLAY / HUD     |  |  AUDIO OUTPUT                        |  |
|  [   ] LIBRARY PATHS     |  |  [ Device: Default Output [v] ]      |  |
|  [   ] ALAYA-LINK (Net)  |  |  [ Latency: 25ms           ]         |  |
|                          |  +--------------------------------------+  |
|                          |                                            |
|                          |  +--------------------------------------+  |
|                          |  |  MICROPHONE CALIBRATION              |  |
|                          |  |  Input Gain:                         |  |
|                          |  |  [=========|=====] 75%               |  |
|                          |  |                                      |  |
|                          |  |  Noise Gate:                         |  |
|                          |  |  [SWITCH: ON] / OFF                  |  |
|                          |  +--------------------------------------+  |
|                          |                                            |
|                          |  +--------------------------------------+  |
|                          |  |  THEME OVERRIDE                      |  |
|                          |  |  ( ) AUTO (System)                   |  |
|                          |  |  (o) TEKKADAN (Dark)                 |  |
|                          |  |  ( ) BARBATOS (Light)                |  |
|                          |  +--------------------------------------+  |
|                          |                                            |
|                          |   [ RESET TO FACTORY ]  [ SAVE CONFIG ]    |
+--------------------------+--------------------------------------------+
````

## 6\. Layout Blueprint: Browsing & Karaoke

### A. Standard Browsing Mode

```text
+-----------------------------------------------------------------------+
|  TopPanel: [Logo] IRON-VOX      [ Library ]  [ Karaoke ]  [ Settings ]|
+-----------------------+-----------------------------------------------+
|  SidePanel            |  CentralPanel                                 |
|                       |                                               |
|  [Search Input   Q]   |  +-----------------------------------------+  |
|                       |  |  HEADER CARD [Chamfered]                |  |
|  MY LIBRARY           |  |  [ Art ]   "Raise Your Flag"            |  |
|  > All Songs          |  |            Man With A Mission           |  |
|  > Favorites          |  |                                         |  |
|  > History            |  |  [ PLAY (Red/Blue) ]  [ Add to Q ]      |  |
|                       |  +-----------------------------------------+  |
|  PLAYLISTS            |                                               |
|  > Anime OPs          |  egui::ScrollArea                             |
|  > Rock Ballads       |  +-----------------------------------------+  |
|                       |  | 01. Survivor             3:42   [Mic]   |  |
|                       |  | 02. Orphans no Namida    4:10   [Mic]   |  |
|                       |  | 03. Fighter              3:50   [Mic]   |  |
|                       |  +-----------------------------------------+  |
|                       |                                               |
+-----------------------+-----------------------------------------------+
|  BottomPanel (Fixed Footer)                                           |
|                                                                       |
|  [Art] Song Title     [<<]  [( > )]  [>>]      [Vol: =======|--]      |
|                                                                       |
|  [------------------========(Handle)-----------------------] 2:10     |
+-----------------------------------------------------------------------+
```

### B. Karaoke Performance Mode (The HUD)

```text
+-----------------------------------------------------------------------+
|  TopPanel: [< Exit]  TARGET: "Rage of Dust"       [ Mic Input |||| ]  |
+-----------------------------------------------------------------------+
|  CentralPanel (Background: Blurred Album Art)                         |
|                                                                       |
|             (Previous Line - Alpha 0.5)                               |
|             Looking for the reason...                                 |
|                                                                       |
|             (Upcoming Line - Alpha 0.5)                               |
|             To fight against the odds...                              |
|                                                                       |
|   [Painter] =======================================================   |
|   [Label]      >>  BETRAYAL IS NOT AN OPTION  <<                      |
|   [Painter] =======================================================   |
|                                                                       |
|             (Next Line - Alpha 0.5)                                   |
|             In this world of steel...                                 |
|                                                                       |
+-----------------------------------------------------------------------+
|  BottomPanel (HUD FOOTER)                                             |
|                                                                       |
|  [ Art ]    01:45 / 03:20                          [ Score: 8740 ]    |
|  [============= (Piston Slider) ===========-----]  [ Skip Intro ]     |
+-----------------------------------------------------------------------+
```

## 7\. Implementation Tips (Rust/egui)

* **Toggles:** Implement the "Mechanical Switch" by drawing a small rectangle that slides inside a larger border. In Dark Mode, make the "ON" state glow Orange (`#D48D3B`).
* **Theme Switching:**

    ```rust
    // Simplified logic
    if self.theme == Theme::Tekkadan {
        ctx.set_visuals(Visuals {
            dark_mode: true,
            window_fill: Color32::from_rgb(17, 19, 17), // Void Green
            panel_fill: Color32::from_rgb(17, 19, 17),
            ..Visuals::dark()
        });
    }
    ```

* **Separators:** Do not use simple lines. Use a custom separator that looks like a "caution stripe" (diagonal lines) for the Settings panel headers.
