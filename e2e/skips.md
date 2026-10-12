# Skipped e2e scenarios

Generated from the `scenario!` skips in `e2e/tests/all` by `cargo test -p e2e --lib skips`; edit the skip, not this file. A reason starting `upstream:` waits on Dioxus, Blitz, wry or the engine (revisit on their update, todo 3362), a number names the todo that removes it.

The performance scenarios (units `perf`, `table_perf`) are no skips but run on request only: the runner leaves them out on CI and `shard.py` assigns them to no shard, unless `E2E_PERF` is set (todo 3454).

| Test | Platform | Reason |
| --- | --- | --- |
| `accordion::focus_returns_to_the_trigger_of_a_panel_that_closes_around_it` | android | 958: element identity on the WebView |
| `accordion::focus_returns_to_the_trigger_of_a_panel_that_closes_around_it` | desktop | 958: element identity on the WebView |
| `accordion::focus_returns_to_the_trigger_of_a_panel_that_closes_around_it` | ios | 3416: focus stays on the body after a Tab or a tap |
| `accordion::the_arrows_move_focus_skip_the_disabled_and_never_toggle` | android | 958: element identity on the WebView |
| `accordion::the_arrows_move_focus_skip_the_disabled_and_never_toggle` | desktop | 958: element identity on the WebView |
| `accordion::the_arrows_move_focus_skip_the_disabled_and_never_toggle` | ios | 3416: the simulator's tap or drag did not drive the page |
| `audio::the_bars_rise_with_the_decoded_tone` | native | Blitz has no Web Audio to decode with |
| `bar_chart::a_click_on_a_bar_reports_it` | ios | 3416: the simulator's tap or drag did not drive the page |
| `bar_chart::a_click_on_a_bar_reports_it` | native | charts are parked on Blitz |
| `bar_chart::a_loading_chart_says_so` | native | charts are parked on Blitz |
| `bar_chart::a_pointer_on_a_line_s_point_picks_it` | ios | 3416: the simulator's tap or drag did not drive the page |
| `bar_chart::a_pointer_on_a_line_s_point_picks_it` | native | charts are parked on Blitz |
| `bar_chart::escape_closes_a_hovered_tooltip_without_focus` | android | a touch screen has no hover |
| `bar_chart::escape_closes_a_hovered_tooltip_without_focus` | ios | a touch screen has no hover |
| `bar_chart::escape_closes_a_hovered_tooltip_without_focus` | native | charts are parked on Blitz |
| `bar_chart::hovering_shows_the_bar_s_tooltip_and_marker` | android | a touch screen has no hover |
| `bar_chart::hovering_shows_the_bar_s_tooltip_and_marker` | ios | a touch screen has no hover |
| `bar_chart::hovering_shows_the_bar_s_tooltip_and_marker` | native | charts are parked on Blitz |
| `bar_chart::lying_bars_zoom_and_brush_down_their_side` | ios | 3416: the simulator's tap or drag did not drive the page |
| `bar_chart::lying_bars_zoom_and_brush_down_their_side` | native | charts are parked on Blitz |
| `bar_chart::the_keys_follow_lying_bars` | native | charts are parked on Blitz |
| `bar_chart::the_keys_read_a_bar_s_range` | native | charts are parked on Blitz |
| `bar_chart::the_keys_walk_onto_a_line_among_bars` | native | charts are parked on Blitz |
| `bar_chart::the_keys_walk_the_bars` | native | charts are parked on Blitz |
| `bar_chart::the_legend_hides_and_shows_a_series` | native | charts are parked on Blitz |
| `bottom_navigation::a_fixed_bar_docks_everywhere` | macos desktop | 3432: WKWebView: the fixed bar never docks |
| `button_group::the_focused_button_is_lifted_over_its_neighbours` | ios | 3416: focus stays on the body after a Tab or a tap |
| `carousel_autoplay::tab_from_pause_within_the_carousel` | android | 2652 is the desktop WebView's arm |
| `carousel_autoplay::tab_from_pause_within_the_carousel` | native | native::carousel has the Blitz arm |
| `carousel_autoplay::tab_from_pause_within_the_carousel` | web | tab_from_pause_to_the_track_is_no_new_entry is the web's arm |
| `cascader::a_disabled_cascader_row_looks_disabled` | ios | a touch screen has no hover |
| `cascader::taps_walk_the_cascader_levels` | ios | 3416: the simulator's tap or drag did not drive the page |
| `cascader_phone::android_back_closes_a_cascader` | all but android | 1275: no Back key off Android |
| `chart::the_segments_swap_the_chart` | native | charts are parked on Blitz |
| `color_field::arrow_down_enters_the_colour_dialog_and_escape_returns` | android | 958: element identity on the WebView |
| `color_field::arrow_down_enters_the_colour_dialog_and_escape_returns` | desktop | 958: element identity on the WebView |
| `color_field::arrow_down_enters_the_colour_dialog_and_escape_returns` | ios | 3416: the simulator's tap or drag did not drive the page |
| `combobox::a_click_on_a_multi_select_option_keeps_the_list_open` | macos desktop | 3432: WKWebView: a click on a listbox option does not register |
| `combobox::a_click_on_a_multi_select_option_toggles_it` | macos desktop | 3432: WKWebView: a click on a listbox option does not register |
| `combobox::a_click_on_an_autocomplete_option_picks_it` | macos desktop | 3432: WKWebView: a click on a listbox option does not register |
| `combobox::clicks_walk_a_cascader_to_a_leaf` | macos desktop | 3432: WKWebView: a click on a listbox option does not register |
| `combobox::typing_and_enter_add_a_tag` | ios | 3416: the simulator's tap or drag did not drive the page |
| `drawer::a_backdrop_click_closes_a_drawer` | ios | 3416: the simulator's tap or drag did not drive the page |
| `drawer::a_backdrop_click_closes_a_drawer` | macos desktop | upstream: WebKit does not focus a clicked button |
| `drawer::a_button_in_a_drawer_closes_it_and_focus_returns` | ios | 3416: the simulator's tap or drag did not drive the page |
| `drawer::a_button_in_a_drawer_closes_it_and_focus_returns` | macos desktop | upstream: WebKit does not focus a clicked button |
| `drawer::a_drawer_moves_focus_in_and_traps_tab` | android | 959: Tab leaves the drawer for the body on the WebView |
| `drawer::a_drawer_moves_focus_in_and_traps_tab` | desktop | 959: Shift+Tab from the first control leaves the drawer on the WebView |
| `drawer::a_drawer_moves_focus_in_and_traps_tab` | ios | 3416: the simulator's tap or drag did not drive the page |
| `drawer::android_back_closes_a_drawer` | all but android | 1275: no Back key off Android |
| `drawer::escape_closes_a_drawer_and_returns_focus` | ios | 3416: the simulator's tap or drag did not drive the page |
| `drawer::escape_closes_a_drawer_and_returns_focus` | macos desktop | upstream: WebKit does not focus a clicked button |
| `drawer::ondismiss_keeps_a_drawer_open` | android | 958: element identity on the WebView |
| `drawer::ondismiss_keeps_a_drawer_open` | ios | 3416: the simulator's tap or drag did not drive the page |
| `drawer::ondismiss_keeps_a_drawer_open` | macos desktop | upstream: WebKit does not focus a clicked button |
| `fcm::android_firebase_answers_a_register` | all but android | 3404: the Kotlin service is Android's |
| `fcm::android_the_service_events_reach_the_handle` | all but android | 3404: the Kotlin service is Android's |
| `field_frame::a_long_word_wraps_inside_the_field` | native | Blitz has no script reads |
| `file_field::browse_opens_the_ios_picker` | all but ios | the system picker's text is read on iOS only |
| `file_field::browse_picks_a_file_and_rust_reads_it` | desktop | 1126: the GTK file chooser is not driven |
| `file_field::browse_picks_a_file_and_rust_reads_it` | ios | 3194: the simulator's picker opens but is not driven; see browse_opens_the_ios_picker |
| `file_field::browse_picks_a_file_and_rust_reads_it` | native | rfd opens the desktop portal's dialog, which no driver answers |
| `file_field::browse_picks_a_second_time` | desktop | 1126: the GTK file chooser is not driven |
| `file_field::browse_picks_a_second_time` | ios | 3194: the simulator's picker opens but is not driven; see browse_opens_the_ios_picker |
| `file_field::browse_picks_a_second_time` | native | rfd opens the desktop portal's dialog, which no driver answers |
| `floating_window::f6_moves_focus_between_the_window_and_the_page` | android | 958: element identity on the WebView |
| `floating_window::f6_moves_focus_between_the_window_and_the_page` | desktop | 958: element identity on the WebView |
| `floating_window::f6_moves_focus_between_the_window_and_the_page` | ios | 3416: the simulator's tap or drag did not drive the page |
| `floating_window::it_meets_the_baseline` | windows web | 3423: the AX value of the resize separator follows the fonts' width |
| `floating_window::the_menu_and_the_step_buttons_meet_the_baseline` | windows web | 3423: the AX value of the resize separator follows the fonts' width |
| `floating_window::the_title_bar_menu_resizes_and_resets` | android | 958: element identity on the WebView |
| `floating_window::the_title_bar_menu_resizes_and_resets` | desktop | 958: element identity on the WebView |
| `floating_window::the_title_bar_menu_resizes_and_resets` | ios | 3416: the simulator's tap or drag did not drive the page |
| `floating_window_resize::a_drag_past_the_edge_keeps_it_in_the_viewport` | ios | 3416: the simulator's tap or drag did not drive the page |
| `floating_window_resize::a_resize_wins_over_the_callers_sx_size` | ios | 3416: the simulator's tap or drag did not drive the page |
| `floating_window_resize::drags_leave_their_handle_focused` | android | 958: element identity on the WebView |
| `floating_window_resize::drags_leave_their_handle_focused` | desktop | 958: element identity on the WebView |
| `floating_window_resize::drags_leave_their_handle_focused` | ios | 3416: the simulator's tap or drag did not drive the page |
| `focus_return::a_modal_opened_by_click_returns_focus_to_its_trigger` | android | 958: element identity on the WebView |
| `focus_return::a_modal_opened_by_click_returns_focus_to_its_trigger` | desktop | 958: element identity on the WebView |
| `focus_return::a_modal_opened_by_click_returns_focus_to_its_trigger` | ios | 3416: focus stays on the body after a Tab or a tap |
| `focus_return::a_modal_opened_by_keyboard_still_returns_focus_to_its_trigger` | android | 958: element identity on the WebView |
| `focus_return::a_modal_opened_by_keyboard_still_returns_focus_to_its_trigger` | desktop | 958: element identity on the WebView |
| `focus_start::a_click_on_prose_moves_the_starting_point_away_from_a_focused_control` | ios | 3416: focus stays on the body after a Tab or a tap |
| `focus_start::a_click_on_prose_moves_the_starting_point_away_from_a_focused_control` | macos desktop | upstream: WebKit does not focus a clicked button |
| `focus_start::a_second_tab_carries_on_from_the_first` | ios | 3416: focus stays on the body after a Tab or a tap |
| `focus_start::shift_tab_after_a_click_starts_from_the_clicked_node` | ios | 3416: focus stays on the body after a Tab or a tap |
| `focus_start::shift_tab_after_the_focused_element_is_removed_starts_where_it_was` | ios | 3416: focus stays on the body after a Tab or a tap |
| `focus_start::tab_after_a_click_removed_its_button_starts_where_it_was` | ios | 3416: focus stays on the body after a Tab or a tap |
| `focus_start::tab_after_a_click_starts_from_the_clicked_node` | ios | 3416: focus stays on the body after a Tab or a tap |
| `focus_start::tab_after_the_focused_element_is_removed_starts_where_it_was` | ios | 3416: focus stays on the body after a Tab or a tap |
| `focus_trap::a_nested_trap_moves_focus_once_per_tab` | android | 958: element identity on the WebView |
| `focus_trap::a_nested_trap_moves_focus_once_per_tab` | desktop | 958: element identity on the WebView |
| `focus_trap::a_nested_trap_moves_focus_once_per_tab` | ios | 3416: the simulator's tap or drag did not drive the page |
| `focus_trap::a_radio_group_is_one_tab_stop` | android | 958: element identity on the WebView |
| `focus_trap::a_radio_group_is_one_tab_stop` | desktop | 958: element identity on the WebView |
| `focus_trap::a_radio_group_is_one_tab_stop` | ios | 3416: focus stays on the body after a Tab or a tap |
| `focus_trap::an_autofocus_target_that_takes_no_focus_falls_through` | android | 958: element identity on the WebView |
| `focus_trap::an_autofocus_target_that_takes_no_focus_falls_through` | desktop | 958: element identity on the WebView |
| `focus_trap::escape_or_release_hands_focus_back_to_the_switch` | android | 958: element identity on the WebView |
| `focus_trap::escape_or_release_hands_focus_back_to_the_switch` | desktop | 958: element identity on the WebView |
| `focus_trap::radio_groups_split_by_form` | android | 958: element identity on the WebView |
| `focus_trap::radio_groups_split_by_form` | desktop | 958: element identity on the WebView |
| `focus_trap::radio_groups_split_by_form` | ios | 3416: the simulator's tap or drag did not drive the page |
| `focus_trap::tab_passes_over_a_stop_that_is_not_rendered` | android | 958: element identity on the WebView |
| `focus_trap::tab_passes_over_a_stop_that_is_not_rendered` | desktop | 958: element identity on the WebView |
| `focus_trap::tab_passes_over_a_stop_that_is_not_rendered` | ios | 3416: the simulator's tap or drag did not drive the page |
| `home::arrow_down_after_typing_enters_on_the_typed_day` | android | 958: element identity on the WebView |
| `home::arrow_down_after_typing_enters_on_the_typed_day` | desktop | 958: element identity on the WebView |
| `home::arrow_down_after_typing_enters_on_the_typed_day` | ios | 3416: the simulator's tap or drag did not drive the page |
| `hover_card::a_long_press_on_a_link_trigger_opens_no_link_menu` | desktop | 1126: no touch input under Xvfb |
| `hover_card::a_long_press_on_a_link_trigger_opens_no_link_menu` | native | 3317: Blitz opens no link menu on a long press, and a script reads its suppression |
| `hover_card::a_long_press_opens_it_on_touch_and_a_tap_does_not` | desktop | 1126: no touch input under Xvfb |
| `hover_card::a_scrolling_text_card_takes_a_tab_stop_that_scrolls_it` | android | 958: the Tab bridge reads the card's focusables inside a handler |
| `hover_card::a_scrolling_text_card_takes_a_tab_stop_that_scrolls_it` | desktop | 958: the Tab bridge reads the card's focusables inside a handler |
| `hover_card::a_scrolling_text_card_takes_a_tab_stop_that_scrolls_it` | ios | 3416: focus stays on the body after a Tab or a tap |
| `hover_card::escape_closes_a_pointer_opened_card_with_focus_elsewhere` | ios | a touch screen has no hover |
| `hover_card::escape_closes_a_select_list_in_the_card_before_the_card` | ios | a touch screen has no hover |
| `hover_card::escape_on_the_trigger_closes_a_pointer_opened_card` | ios | a touch screen has no hover |
| `hover_card::tab_crosses_into_the_portaled_card_and_back_on_every_backend` | android | 958: the Tab bridge reads the card's focusables inside a handler |
| `hover_card::tab_crosses_into_the_portaled_card_and_back_on_every_backend` | desktop | 958: the Tab bridge reads the card's focusables inside a handler |
| `hover_card::tab_crosses_into_the_portaled_card_and_back_on_every_backend` | ios | 3416: focus stays on the body after a Tab or a tap |
| `hover_card::tab_crosses_past_a_hidden_last_control_on_the_trigger_and_the_card` | android | 958: the Tab bridge reads the card's focusables inside a handler |
| `hover_card::tab_crosses_past_a_hidden_last_control_on_the_trigger_and_the_card` | desktop | 958: the Tab bridge reads the card's focusables inside a handler |
| `hover_card::tab_crosses_past_a_hidden_last_control_on_the_trigger_and_the_card` | ios | 3416: focus stays on the body after a Tab or a tap |
| `hover_card::tab_enters_the_card_only_from_the_triggers_last_focusable` | android | 958: the Tab bridge reads the card's focusables inside a handler |
| `hover_card::tab_enters_the_card_only_from_the_triggers_last_focusable` | desktop | 958: the Tab bridge reads the card's focusables inside a handler |
| `hover_card::tab_enters_the_card_only_from_the_triggers_last_focusable` | ios | 3416: focus stays on the body after a Tab or a tap |
| `hover_card::tab_walks_the_card_in_positive_tabindex_order` | android | 958: the Tab bridge reads the card's focusables inside a handler |
| `hover_card::tab_walks_the_card_in_positive_tabindex_order` | desktop | 958: the Tab bridge reads the card's focusables inside a handler |
| `hover_card::tab_walks_the_card_in_positive_tabindex_order` | ios | 3416: focus stays on the body after a Tab or a tap |
| `hover_card::the_pointer_crosses_into_the_card_and_leaving_closes_it` | ios | a touch screen has no hover |
| `image_cropper::a_finger_beside_the_box_moves_it` | all but android | 2123: the web and Blitz arms are a_drag_or_wheel_beside_the_box_moves_or_scales_it |
| `image_cropper::a_picked_image_is_cropped_through_the_page` | native | Blitz runs no script to stub the picker; `crop_bytes` unit tests cover the cut |
| `kanban::a_filtered_button_move_lands_before_the_next_shown_card_behind_a_hidden_one` | desktop | 958: element identity on the WebView |
| `kanban::a_filtered_column_moves_by_keys_and_buttons_over_the_shown_cards` | desktop | 958: element identity on the WebView |
| `kanban::a_filtered_column_still_drags_and_move_to_lands_after_its_last_card` | desktop | 958: element identity on the WebView |
| `kanban::a_filtered_drag_lands_before_the_next_shown_card_behind_a_hidden_one` | desktop | 958: element identity on the WebView |
| `kanban::a_filtered_key_move_lands_before_the_next_shown_card_behind_a_hidden_one` | desktop | 958: element identity on the WebView |
| `kanban::a_handle_drag_moves_a_card_into_an_empty_column` | ios | 3416: flaky: the drop at the screen edge races the board's auto-scroll |
| `kanban::a_handle_drag_moves_a_card_into_the_empty_last_column_of_a_right_to_left_board` | ios | 3416: flaky: the drop at the screen edge races the board's auto-scroll |
| `kanban::a_handle_drag_moves_a_card_into_the_empty_last_column_of_a_right_to_left_board` | native | 707: Blitz has no right-to-left scroll |
| `kanban::a_handle_drag_moves_a_card_leftwards_into_the_next_column_of_a_right_to_left_board` | native | 707: Blitz has no right-to-left scroll |
| `kanban::a_handle_drag_reorders_a_column_of_a_right_to_left_board` | native | 707: Blitz has no right-to-left scroll |
| `kanban::a_header_handle_drag_reorders_the_columns_of_a_right_to_left_board` | macos desktop | upstream: WebKit orders the RTL columns unlike WebKitGTK |
| `kanban::a_header_handle_drag_reorders_the_columns_of_a_right_to_left_board` | native | 707: Blitz has no right-to-left scroll |
| `kanban::a_move_the_app_refuses_leaves_no_focus_jump_for_a_later_card` | desktop | 958: element identity on the WebView |
| `kanban::a_refused_move_is_void_once_the_app_removes_its_card` | desktop | 958: element identity on the WebView |
| `kanban::a_refused_move_is_void_when_the_data_change_renders_only_a_column` | desktop | 958: element identity on the WebView |
| `kanban::a_right_to_left_board_lays_its_columns_from_the_right` | native | 707: Blitz has no right-to-left scroll |
| `kanban::a_same_column_key_or_button_move_the_app_refuses_is_not_announced` | desktop | 958: element identity on the WebView |
| `kanban::left_and_right_put_a_lifted_card_in_the_next_column_of_a_right_to_left_board_mirrored` | macos desktop | upstream: WebKit orders the RTL columns unlike WebKitGTK |
| `kanban::left_and_right_put_a_lifted_card_in_the_next_column_of_a_right_to_left_board_mirrored` | native | 707: Blitz has no right-to-left scroll |
| `kanban::left_moves_a_lifted_column_forward_on_a_right_to_left_board` | macos desktop | upstream: WebKit orders the RTL columns unlike WebKitGTK |
| `kanban::left_moves_a_lifted_column_forward_on_a_right_to_left_board` | native | 707: Blitz has no right-to-left scroll |
| `kanban::left_or_right_moves_no_resting_card_and_a_refused_move_is_not_announced` | desktop | 958: element identity on the WebView |
| `kanban::the_move_to_menu_lists_every_column_with_the_cards_own_disabled` | desktop | 958: element identity on the WebView |
| `kanban::the_move_to_menu_lists_every_column_with_the_cards_own_disabled` | ios | 3416: focus stays on the body after a Tab or a tap |
| `kanban::the_move_to_menu_moves_a_card_across_a_right_to_left_board` | desktop | 958: element identity on the WebView |
| `kanban::the_move_to_menu_moves_a_card_across_a_right_to_left_board` | native | 707: Blitz has no right-to-left scroll |
| `kanban::the_move_to_menu_moves_a_card_to_the_end_of_another_column_and_keeps_the_focus` | desktop | 958: element identity on the WebView |
| `lightbox_zoom::a_pinch_zooms_the_picture` | desktop | 1126: no touch input under Xvfb |
| `lightbox_zoom::a_pinch_zooms_the_picture` | ios | 3416: the simulator's tap or drag did not drive the page |
| `lightbox_zoom::a_pinch_zooms_the_picture` | native | Blitz takes no multi-touch input |
| `lightbox_zoom::a_tap_on_zoom_in_zooms_the_picture` | macos desktop | 3432: WKWebView: the stage's first scroll never settles |
| `lightbox_zoom::a_zoomed_picture_pans_only_to_its_edge` | macos desktop | 3432: WKWebView: the stage's first scroll never settles |
| `lightbox_zoom::a_zoomed_picture_pans_only_to_its_edge` | native | 3386: reads the picture's size and pan by script, which Blitz has none of |
| `lightbox_zoom::a_zoomed_picture_pans_only_to_its_edge` | windows desktop | upstream: Dioxus' asset! gives a backslashed file path (D:\a\..\1.svg) off the bundle, which WebView2 cannot load, so no picture decodes |
| `line_chart::a_click_on_a_point_reports_it` | ios | 3416: the simulator's tap or drag did not drive the page |
| `line_chart::a_click_on_a_point_reports_it` | native | charts are parked on Blitz |
| `line_chart::a_pen_leaving_takes_its_tooltip_along` | android | a synthetic pen hover is a browser check |
| `line_chart::a_pen_leaving_takes_its_tooltip_along` | ios | a synthetic pen hover is a browser check |
| `line_chart::a_pen_leaving_takes_its_tooltip_along` | native | charts are parked on Blitz |
| `line_chart::a_pinch_zooms_the_plot` | desktop | a mouse has no pinch |
| `line_chart::a_pinch_zooms_the_plot` | ios | 3416: the simulator's tap or drag did not drive the page |
| `line_chart::a_pinch_zooms_the_plot` | native | charts are parked on Blitz |
| `line_chart::a_pinch_zooms_the_plot` | web | a mouse has no pinch |
| `line_chart::a_sideways_drag_brushes_a_range` | ios | 3416: the simulator's tap or drag did not drive the page |
| `line_chart::a_sideways_drag_brushes_a_range` | native | charts are parked on Blitz |
| `line_chart::a_vertical_swipe_over_the_plot_scrolls_the_page` | desktop | a mouse drag scrolls nothing |
| `line_chart::a_vertical_swipe_over_the_plot_scrolls_the_page` | native | charts are parked on Blitz |
| `line_chart::a_vertical_swipe_over_the_plot_scrolls_the_page` | web | a mouse drag scrolls nothing |
| `line_chart::hovering_shows_the_point_s_tooltip_and_marker` | android | a touch screen has no hover |
| `line_chart::hovering_shows_the_point_s_tooltip_and_marker` | ios | a touch screen has no hover |
| `line_chart::hovering_shows_the_point_s_tooltip_and_marker` | native | charts are parked on Blitz |
| `line_chart::the_chart_exports_as_svg_and_png` | native | charts are parked on Blitz |
| `line_chart::the_keys_walk_the_points` | native | charts are parked on Blitz |
| `line_chart::the_keys_zoom_pan_and_brush` | native | charts are parked on Blitz |
| `line_chart::the_legend_hides_and_shows_a_line` | native | charts are parked on Blitz |
| `line_chart::the_tooltip_keeps_its_height_from_point_to_point` | native | charts are parked on Blitz |
| `menu::a_click_outside_closes_it_and_leaves_focus_where_it_went` | ios | 3416: the simulator's tap or drag did not drive the page |
| `menu::a_click_outside_closes_it_and_leaves_focus_where_it_went` | macos desktop | upstream: WebKit does not focus a clicked button |
| `menu::a_tall_menu_above_fits_the_room` | macos desktop | 3432: WKWebView: the last item never scrolls into view |
| `menu::a_tall_menu_above_fits_the_room` | windows desktop | 3421: END scrolls by the listbox height from before the popup shrank |
| `menu::a_tall_menu_below_fits_the_room` | macos desktop | 3432: WKWebView: a wheel scroll inside the menu never lands |
| `menu::a_tall_menu_below_fits_the_room` | windows desktop | 3421: END scrolls by the listbox height from before the popup shrank |
| `menubar::hovering_another_trigger_switches_the_open_menu` | ios | a touch screen has no hover |
| `menubar::the_arrows_rove_along_the_bar_and_wrap` | android | 958: element identity on the WebView |
| `menubar::the_arrows_rove_along_the_bar_and_wrap` | desktop | 958: element identity on the WebView |
| `menubar::the_arrows_rove_along_the_bar_and_wrap` | ios | 3416: the simulator's tap or drag did not drive the page |
| `modal::a_backdrop_click_closes_a_modal` | android | 958: element identity on the WebView |
| `modal::a_backdrop_click_closes_a_modal` | ios | 3416: the simulator's tap or drag did not drive the page |
| `modal::a_backdrop_click_closes_a_modal` | macos desktop | upstream: WebKit does not focus a clicked button |
| `modal::a_modal_moves_focus_in_and_traps_tab` | android | 958: element identity on the WebView |
| `modal::a_modal_moves_focus_in_and_traps_tab` | desktop | 958: element identity on the WebView |
| `modal::a_modal_moves_focus_in_and_traps_tab` | ios | 3416: the simulator's tap or drag did not drive the page |
| `modal::a_modal_takes_focus_and_the_first_escape_closes_it` | ios | 3416: the simulator's tap or drag did not drive the page |
| `modal::a_modal_takes_focus_and_the_first_escape_closes_it` | macos desktop | upstream: WebKit does not focus a clicked button |
| `modal::an_alertdialog_ignores_the_backdrop` | android | 958: element identity on the WebView |
| `modal::an_alertdialog_ignores_the_backdrop` | ios | 3416: the simulator's tap or drag did not drive the page |
| `modal::an_alertdialog_ignores_the_backdrop` | macos desktop | upstream: WebKit does not focus a clicked button |
| `modal::android_back_closes_the_top_layer` | all but android | 1275: no Back key off Android |
| `modal::android_taps_in_a_dialog_leave_no_back_entry` | all but android | 3322: no Back key off Android |
| `modal::content_that_is_no_dialog_takes_clicks` | ios | 3416: the simulator's tap or drag did not drive the page |
| `modal::content_that_is_no_dialog_takes_clicks` | macos desktop | upstream: WebKit does not focus a clicked button |
| `modal::content_that_is_no_dialog_takes_focus_on_its_box` | ios | 3416: the simulator's tap or drag did not drive the page |
| `modal::content_that_is_no_dialog_takes_focus_on_its_box` | macos desktop | upstream: WebKit does not focus a clicked button |
| `modal::escape_in_a_modal_menu_closes_only_the_menu` | android | 958: element identity on the WebView |
| `modal::escape_in_a_modal_menu_closes_only_the_menu` | desktop | 958: element identity on the WebView |
| `modal::escape_in_a_modal_menu_closes_only_the_menu` | ios | 3416: the simulator's tap or drag did not drive the page |
| `modal::ondismiss_keeps_a_draft_open` | android | 958: element identity on the WebView |
| `modal::ondismiss_keeps_a_draft_open` | ios | 3416: the simulator's tap or drag did not drive the page |
| `modal::ondismiss_keeps_a_draft_open` | macos desktop | upstream: WebKit does not focus a clicked button |
| `modal::tab_cycles_through_every_button_in_a_modal` | android | 959: the trap cannot wrap Tab on a WebView |
| `modal::tab_cycles_through_every_button_in_a_modal` | desktop | 959: the trap cannot wrap Tab on a WebView |
| `modal::tab_cycles_through_every_button_in_a_modal` | ios | 3416: the simulator's tap or drag did not drive the page |
| `modal::the_trap_follows_positive_tabindex` | android | 958: element identity on the WebView |
| `modal::the_trap_follows_positive_tabindex` | desktop | 958: element identity on the WebView |
| `modal::the_trap_follows_positive_tabindex` | ios | 3416: focus stays on the body after a Tab or a tap |
| `multi_select::the_chips_take_the_focus_and_remove` | native | 3406: Blitz hands a removed chip's focus to a container div, parked with native |
| `native_select::the_arrows_step_a_focused_native_select` | desktop | WebKitGTK opens the picker on an arrow |
| `native_select::the_arrows_step_a_focused_native_select` | ios | WKWebView opens the picker on an arrow |
| `native_select::the_arrows_step_a_focused_native_select` | native | a listbox Select stands in for the \<select> on Blitz |
| `notifications::f8_focuses_the_newest_notification_on_the_event_thread` | all but android | 2216: the split event thread is Android's; the web test above covers F8 |
| `pagination::a_disabling_arrow_hands_focus_to_the_current_page` | android | 958: element identity on the WebView |
| `pagination::a_disabling_arrow_hands_focus_to_the_current_page` | desktop | 958: element identity on the WebView |
| `pagination::a_disabling_arrow_hands_focus_to_the_current_page` | ios | 3416: focus stays on the body after a Tab or a tap |
| `pagination::a_late_answer_hands_focus_to_the_current_page_too` | android | 958: element identity on the WebView |
| `pagination::a_late_answer_hands_focus_to_the_current_page_too` | desktop | 958: element identity on the WebView |
| `pagination::a_late_answer_hands_focus_to_the_current_page_too` | ios | 3416: focus stays on the body after a Tab or a tap |
| `phone_field::a_picker_the_keyboard_covers_scrolls_back_with_its_list` | android | the keyboard's height is the device's: the first scenario covers it |
| `phone_field::a_picker_the_keyboard_covers_scrolls_back_with_its_list` | desktop | no soft keyboard, and wry's window is not resized here |
| `phone_field::a_picker_the_keyboard_covers_scrolls_back_with_its_list` | ios | 3416: the simulator's tap or drag did not drive the page |
| `phone_field::a_picker_the_keyboard_covers_scrolls_back_with_its_list` | native | Blitz reports no window resize (todo 2129) |
| `phone_field::the_country_list_fits_a_viewport_the_keyboard_shrank` | desktop | no soft keyboard, and wry's window is not resized here |
| `phone_field::the_country_list_fits_a_viewport_the_keyboard_shrank` | ios | 3416: the simulator's tap or drag did not drive the page |
| `phone_field::the_country_list_fits_a_viewport_the_keyboard_shrank` | native | Blitz reports no window resize (todo 2129) |
| `picker_dialog::a_tap_on_a_date_range_field_opens_only_the_calendar` | ios | 3416: the simulator's tap or drag did not drive the page |
| `picker_dialog::android_back_closes_a_color_field` | all but android | 1275: no Back key off Android |
| `picker_dialog::android_back_closes_a_date_field` | all but android | 1275: no Back key off Android |
| `pie_chart::a_click_on_a_slice_reports_it` | ios | 3416: the simulator's tap or drag did not drive the page |
| `pie_chart::a_click_on_a_slice_reports_it` | native | charts are parked on Blitz |
| `pie_chart::a_hidden_slice_grows_the_shares_the_eye_reads_not_the_summary` | native | charts are parked on Blitz |
| `pie_chart::the_keys_walk_the_slices` | native | charts are parked on Blitz |
| `pie_chart::the_legend_hides_and_shows_a_slice` | native | charts are parked on Blitz |
| `pin_field::a_digit_retyped_without_a_key_leaves_no_drifted_cell` | desktop | 1126: no text without a key (no IME path) under xdotool |
| `pin_field::a_digit_retyped_without_a_key_leaves_no_drifted_cell` | ios | 3416: no text without a key through AXe |
| `pin_field::a_digit_retyped_without_a_key_leaves_no_drifted_cell` | native | no text without a key on Blitz |
| `pin_field::a_pin_field_takes_one_digit_per_cell_and_moves_on` | macos native | 3342: Blitz deletes twice on AppKit's Backspace binding |
| `pin_field::the_rule_waits_for_focus_to_leave_the_cells` | android | 1564: a WebView cannot tell a move inside the field from a blur |
| `pin_field::the_rule_waits_for_focus_to_leave_the_cells` | desktop | 1564: a WebView cannot tell a move inside the field from a blur |
| `pin_field::the_rule_waits_for_focus_to_leave_the_cells` | ios | 3416: the simulator's tap or drag did not drive the page |
| `pin_field::the_rule_waits_for_focus_to_leave_the_cells` | web | the_rule_waits_until_focus_leaves_the_cells checks it after two frames |
| `platform_probe::the_platform_probe_records_each_engine` | native | Blitz runs no script |
| `popover::a_field_dropdown_opened_inside_counts_as_inside` | android | 1425: no synchronous containment on the WebView |
| `popover::a_field_dropdown_opened_inside_counts_as_inside` | desktop | 1425: no synchronous containment on the WebView |
| `popover::a_field_dropdown_opened_inside_counts_as_inside` | ios | 3416: the simulator's tap or drag did not drive the page |
| `popover::android_back_closes_a_popover` | all but android | 1275: no Back key off Android |
| `rating::a_key_press_over_a_hover_draws_the_new_value` | android | a touch screen has no hover |
| `rating::a_key_press_over_a_hover_draws_the_new_value` | ios | a touch screen has no hover |
| `rating::hovering_previews_without_changing_the_value` | android | a touch screen has no hover |
| `rating::hovering_previews_without_changing_the_value` | ios | a touch screen has no hover |
| `refused::a_press_on_a_disabled_button_focuses_what_holds_it` | ios | 3416: focus stays on the body after a Tab or a tap |
| `refused::a_press_on_a_disabled_button_focuses_what_holds_it` | macos desktop | upstream: WebKit does not focus a clicked button |
| `refused::tab_skips_disabled_buttons_and_an_inert_subtree` | android | 958: element identity on the WebView |
| `save_file::android_shares_binary_bytes_as_a_file` | all but android | 2078: the share sheet is Android's |
| `save_file::android_shares_text_through_the_share_sheet` | all but android | 2016: the share sheet is Android's |
| `scroll_area::a_jump_mid_list_renders_the_window_there` | android | 2177: the WebView does not follow the jump |
| `scroll_area::rows_stay_in_view_through_a_fling` | android | 2131: rows trail a WebView fling; skeleton bars paint the gap |
| `scroll_area::rows_stay_in_view_through_a_fling` | desktop | 2131: rows trail a WebView fling, 80px of 120; skeleton bars paint the gap |
| `scroll_area::rows_stay_in_view_through_a_fling` | ios | 3416: the simulator's tap or drag did not drive the page |
| `scroll_area::the_window_follows_a_wheel_or_a_fling` | ios | a touch screen has no wheel |
| `scroll_area::the_window_follows_a_wheel_or_a_fling` | macos desktop | 3432: WKWebView: the wheel never brings the windowed list to rest |
| `segmented_control::the_arrows_move_focus_when_the_id_is_no_css_identifier` | android | 958: element identity on the WebView |
| `segmented_control::the_arrows_move_focus_when_the_id_is_no_css_identifier` | desktop | 958: element identity on the WebView |
| `segmented_control::the_arrows_move_focus_with_the_selection_and_wrap` | android | 958: element identity on the WebView |
| `segmented_control::the_arrows_move_focus_with_the_selection_and_wrap` | desktop | 958: element identity on the WebView |
| `segmented_control::the_arrows_move_focus_with_the_selection_and_wrap` | web | 1826: it_honours_the_radio_group_contract runs these keys at both viewports |
| `select::a_click_on_a_select_option_picks_it` | macos desktop | 3432: WKWebView: a click on a listbox option does not register |
| `select::a_hovered_select_option_tints_like_the_keyboard_s_row` | ios | a touch screen has no hover |
| `select::a_tall_select_below_fits_the_room` | windows desktop | 3421: END scrolls by the listbox height from before the popup shrank |
| `select::android_back_closes_a_select` | all but android | 1275: no Back key off Android |
| `share::android_shares_a_file_through_the_share_sheet` | all but android | 3351: the JNI share sheet is Android's |
| `share::android_shares_the_link_through_the_share_sheet` | all but android | 3351: the JNI share sheet is Android's |
| `share::android_shares_two_files_through_the_share_sheet` | all but android | 3361: ACTION_SEND_MULTIPLE is Android's |
| `shortcut_help::a_narrow_list_stacks_each_row` | native | 1224: Blitz drops `@container`, so the rows stay side by side |
| `slider::a_drag_released_past_the_end_tracks_and_ends` | ios | a touch screen has no hover |
| `slider::a_short_drag_ending_on_the_thumb_holds_and_the_next_tap_lands` | ios | 3416: the simulator's tap or drag did not drive the page |
| `slider::a_vertical_swipe_scrolls_the_page_on_android_only_and_a_sideways_drag_or_a_tap_moves_it` | ios | 3416: the simulator's tap or drag did not drive the page |
| `slider::pressing_the_track_centre_and_dragging_right_follows_the_pointer` | ios | 3416: the simulator's tap or drag did not drive the page |
| `slider_touch_keyboard::a_touch_held_on_the_thumb_leaves_no_bubble_after_the_release` | desktop | 1126: no touch input under Xvfb |
| `sortable::a_click_on_a_hook_handle_clicks_and_a_drag_does_not` | ios | 3416: the simulator's tap or drag did not drive the page |
| `sortable::a_dropped_item_settles_into_its_slot_until_the_next_lift` | ios | 3416: the simulator's tap or drag did not drive the page |
| `sortable::the_neighbours_of_a_lifted_item_slide_and_the_lifted_item_does_not` | macos desktop | 3432: no transition runs on the runner (Reduce Motion?) |
| `splitter::a_double_click_collapses_pane_a_and_restores_it` | ios | 3416: the simulator's tap or drag did not drive the page |
| `splitter::a_vertical_swipe_over_a_vertical_divider_scrolls_the_page_on_android_only_and_a_sideways_drag_moves_it` | ios | 3416: the simulator's tap or drag did not drive the page |
| `spotlight::a_hovered_row_is_tinted_and_does_not_move_the_highlight` | ios | 3416: the simulator's tap or drag did not drive the page |
| `stepper::a_clicked_continue_returns_focus_horizontally` | android | 958: element identity on the WebView |
| `stepper::a_clicked_continue_returns_focus_horizontally` | desktop | 958: element identity on the WebView |
| `stepper::a_clicked_continue_returns_focus_horizontally` | ios | 3416: focus stays on the body after a Tab or a tap |
| `stepper::a_clicked_continue_returns_focus_vertically` | android | 958: element identity on the WebView |
| `stepper::a_clicked_continue_returns_focus_vertically` | desktop | 958: element identity on the WebView |
| `stepper::a_clicked_continue_returns_focus_vertically` | ios | 3416: focus stays on the body after a Tab or a tap |
| `stepper::moving_on_returns_focus_to_the_current_step_horizontally` | android | 958: element identity on the WebView |
| `stepper::moving_on_returns_focus_to_the_current_step_horizontally` | desktop | 958: element identity on the WebView |
| `stepper::moving_on_returns_focus_to_the_current_step_horizontally` | ios | 3416: focus stays on the body after a Tab or a tap |
| `stepper::moving_on_returns_focus_to_the_current_step_vertically` | android | 958: element identity on the WebView |
| `stepper::moving_on_returns_focus_to_the_current_step_vertically` | desktop | 958: element identity on the WebView |
| `stepper::moving_on_returns_focus_to_the_current_step_vertically` | ios | 3416: focus stays on the body after a Tab or a tap |
| `table::a_paged_table_turns_sorts_and_resizes` | macos desktop | 3432: WKWebView: the page range reads 7-7 of 7 after page 5 |
| `table::a_row_click_reports_its_row` | ios | a touch screen has no hover |
| `table::rows_select_by_box_and_keep_it_through_a_sort` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_column_filters::a_date_filter_narrows_a_table` | android | Chromium's typed date segments only |
| `table_column_filters::a_date_filter_narrows_a_table` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_column_filters::a_date_filter_narrows_a_table` | macos desktop | 3432: WKWebView: ArrowDown does not open a select filter's list |
| `table_column_filters::a_date_filter_narrows_a_table` | windows desktop | 3422: Shift+Tab never leaves the date input on WebView2 |
| `table_column_filters::tab_past_either_end_of_a_column_filter_closes_it` | ios | 3416: focus stays on the body after a Tab or a tap |
| `table_column_filters::the_column_filters_narrow_a_table` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_column_filters::the_column_filters_narrow_a_table` | macos desktop | 3432: WKWebView: ArrowDown does not open a select filter's list |
| `table_column_filters::the_filter_panel_edits_every_filter` | android | 958: element identity on the WebView |
| `table_column_filters::the_filter_panel_edits_every_filter` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_column_filters::the_filter_panel_edits_every_filter` | macos desktop | 3432: WKWebView: ArrowDown does not open a select filter's list |
| `table_column_filters::the_quick_filter_narrows_a_paged_table` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_detail::an_animated_detail_slides_open_and_shut` | android | 958: element identity on the WebView |
| `table_detail::pinned_toggles_push_the_checkboxes_past_them` | android | 3217: one web run guards the cascade |
| `table_detail::pinned_toggles_push_the_checkboxes_past_them` | desktop | 3217: one web run guards the cascade |
| `table_detail::pinned_toggles_push_the_checkboxes_past_them` | native | 3217: one web run guards the cascade |
| `table_detail::the_keyboard_opens_a_detail_and_tabs_into_it` | android | 958: element identity on the WebView |
| `table_filter::a_select_filter_narrows_a_table` | android | 958: element identity on the WebView |
| `table_filter::a_select_filter_narrows_a_table` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_filter::a_select_filter_narrows_a_table` | macos desktop | 3432: WKWebView: ArrowDown does not open a select filter's list |
| `table_grid::a_custom_editor_clears_the_error_by_changed` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::a_second_tap_opens_the_editor` | desktop | no touch input in the desktop driver |
| `table_grid::a_second_tap_opens_the_editor` | ios | 3416: focus stays on the body after a Tab or a tap |
| `table_grid::a_second_tap_opens_the_editor` | native | 3242: Blitz fires no pointer events for a touch |
| `table_grid::a_sort_follows_the_apps_language` | android | 3294: the WebView app has no Intl.Collator to call |
| `table_grid::a_sort_follows_the_apps_language` | desktop | 3400: the app sorts in Rust by accent-folded letters, only the web build has an Intl.Collator |
| `table_grid::a_sort_follows_the_apps_language` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_grid::a_sort_follows_the_apps_language` | native | 3386: reads the page language by script, Blitz has none and no Intl.Collator |
| `table_grid::a_windowed_grid_scrolls_to_the_focused_cell` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::a_yes_no_editor_is_a_checkbox_and_a_date_editor_names_its_form` | native | 3386: reads the editor's checked property by script, which Blitz has none of |
| `table_grid::an_arrow_leaves_a_widget_for_the_next_cell` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::an_error_shows_in_a_fixed_row_height_table` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::an_error_shows_in_a_fixed_row_height_table` | native | no script reads |
| `table_grid::arrows_move_the_cell_stop` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::enter_commits_and_escape_cancels` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::enter_on_a_button_in_an_editable_cell_opens_no_editor` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::enter_on_a_header_sorts` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::enter_on_a_sort_button_sorts_once` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::home_end_and_ctrl_home_end` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::page_down_moves_a_view` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::shift_space_selects_the_row` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::space_on_the_select_all_box_acts_once` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::tab_and_shift_enter_move_the_stop` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::tab_enters_and_leaves_in_one_stop` | android | 958: roving keys are inert in the emulator's WebView |
| `table_grid::the_editor_opens_with_its_text_selected` | android | 958: roving keys are inert in the emulator's WebView |
| `table_groups::a_focused_windowed_row_clears_the_footer` | android | 958: element identity on the WebView |
| `table_groups::a_focused_windowed_row_clears_the_footer` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_groups::a_footer_lines_up_under_its_columns_and_sticks` | android | 958: element identity on the WebView |
| `table_groups::a_grouped_header_spans_its_columns_and_sticks` | android | 958: element identity on the WebView |
| `table_groups::a_grouped_header_spans_its_columns_and_sticks` | native | Blitz scrolls on no key and sticks the last header row only: tests/native/table_groups.rs |
| `table_groups::a_pinned_group_holds_while_the_body_scrolls_sideways` | android | 958: element identity on the WebView |
| `table_groups::a_windowed_footer_holds_and_ends_the_rows` | android | 958: element identity on the WebView |
| `table_groups::a_windowed_footer_holds_and_ends_the_rows` | ios | a touch screen has no wheel |
| `table_groups::a_windowed_footer_holds_and_ends_the_rows` | macos desktop | 3432: WKWebView: the wheel never brings the windowed list to rest |
| `table_groups::a_windowed_scrollbar_stops_above_the_footer` | android | 958: element identity on the WebView |
| `table_groups::a_windowed_scrollbar_stops_above_the_footer` | native | 1010: Blitz paints its own bar, ours is not drawn |
| `table_pins::narrow_areas_release_their_pins_and_wide_ones_pin_again` | desktop | its window cannot be narrowed |
| `table_pins::narrow_areas_release_their_pins_and_wide_ones_pin_again` | native | 1024px is wide enough for the pins |
| `table_reorder::a_column_menu_moves_a_column_and_its_sort` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_reorder::a_deep_move_up_keeps_scroll_and_focus` | android | 958: element identity on the WebView |
| `table_reorder::a_deep_move_up_keeps_scroll_and_focus` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_reorder::a_header_grip_drag_moves_a_column` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_reorder::a_move_button_moves_its_row` | android | 958: element identity on the WebView |
| `table_reorder::a_move_to_either_end_hands_the_focus_on` | android | 958: element identity on the WebView |
| `table_reorder::a_pin_or_hide_from_the_menu_keeps_focus_on_a_menu_button` | android | 958: element identity on the WebView |
| `table_reorder::a_windowed_table_reorders_past_its_window` | android | 958: element identity on the WebView |
| `table_reorder::an_early_keyboard_drop_keeps_its_focus` | android | 958: element identity on the WebView |
| `table_reorder::an_early_keyboard_drop_keeps_its_focus` | windows desktop | a drop made before the window follows the scroll leaves the focus on the body in WebView2 |
| `table_reorder::an_open_detail_drags_with_its_row` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_reorder::header_grip_drags_move_columns_twice` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_reorder::hiding_a_column_from_its_own_columns_submenu_keeps_focus` | android | 958: element identity on the WebView |
| `table_reorder::row_reorder_is_off_while_sorted` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_reorder::the_column_menu_announces_in_german` | android | 958: element identity on the WebView |
| `table_reorder::the_keyboard_lifts_and_moves_a_row` | android | 958: element identity on the WebView |
| `table_resize::a_header_grip_drags_the_width_within_its_limits` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_resize::the_column_menu_widens_narrows_and_resets_a_width` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_resize::the_keys_resize_a_column_from_its_grip` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_row_groups::a_windowed_grouped_table_renders_the_rows_in_view` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_scroll::a_capped_table_scrolls_from_its_header` | android | 958: element identity on the WebView |
| `table_scroll::a_focused_row_control_is_not_hidden_by_the_sticky_header` | android | 958: element identity on the WebView |
| `table_scroll::a_plain_capped_table_scrolls_as_a_named_region` | android | 958: element identity on the WebView |
| `table_scroll::a_plain_capped_table_scrolls_as_a_named_region` | macos desktop | 3432: WKWebView: a wheel step never scrolls the capped rows |
| `table_scroll::a_tabbed_header_scrolls_into_its_region` | android | 958: element identity on the WebView |
| `table_scroll::a_tabbed_header_scrolls_into_its_region` | ios | 3416: focus stays on the body after a Tab or a tap |
| `table_scroll::a_windowed_table_renders_the_rows_in_view` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_scroll::a_windowed_table_renders_the_rows_in_view` | macos native | upstream: Blitz's stylo panics in resolve_map on the windowed rows (a SIGSEGV in the full run) |
| `table_scroll::a_windowed_table_renders_the_rows_in_view` | windows native | upstream: Blitz's stylo panics in calc.rs on the windowed rows |
| `table_scroll::end_at_the_bottom_loads_more_rows` | ios | 3416: the simulator's tap or drag did not drive the page |
| `table_tree::a_tree_box_selects_its_descendants_and_mixes_its_ancestors` | ios | 3416: flaky: a tap on a tree box sometimes leaves a half selection |
| `table_tree::a_tree_with_cell_focus_moves_between_rows_and_cells` | android | 958: roving keys are inert in the emulator's WebView |
| `table_tree::the_arrows_move_and_open_the_tree_rows` | android | 958: roving keys are inert in the emulator's WebView |
| `table_tree::the_arrows_move_through_a_windowed_tree` | android | 958: roving keys are inert in the emulator's WebView |
| `tabs::arrow_left_wraps_and_home_and_end_jump` | android | 958: element identity on the WebView |
| `tabs::arrow_left_wraps_and_home_and_end_jump` | desktop | 958: element identity on the WebView |
| `tabs::arrow_right_moves_focus_and_selection_to_the_next_tab` | android | 958: element identity on the WebView |
| `tabs::arrow_right_moves_focus_and_selection_to_the_next_tab` | desktop | 958: element identity on the WebView |
| `tabs::shift_tab_leaves_the_strip_from_a_clicked_disabled_tab` | desktop | 958: element identity on the WebView (Shift+Tab stays in the strip) |
| `tabs::tab_leaves_the_strip_and_shift_tab_comes_back_to_the_selected_tab` | android | 958: element identity on the WebView |
| `tabs::tab_leaves_the_strip_and_shift_tab_comes_back_to_the_selected_tab` | desktop | 958: element identity on the WebView |
| `tabs::tab_leaves_the_strip_from_a_focused_unselected_tab` | android | 958: element identity on the WebView |
| `tabs::tab_leaves_the_strip_from_a_focused_unselected_tab` | desktop | 958: element identity on the WebView |
| `text_field::fast_typing_into_a_rewriting_field_keeps_every_letter` | native | Blitz has no live value read; 1051 is a WebView race |
| `text_field::fast_typing_into_a_text_field_keeps_every_letter` | ios | 3416: the simulator's tap or drag did not drive the page |
| `textarea::typing_into_a_textarea_reaches_its_value` | ios | 3416: the simulator's tap or drag did not drive the page |
| `theme_defaults::an_unset_hover_card_align_follows_the_theme` | ios | a touch screen has no hover |
| `theme_defaults::an_unset_hover_card_side_follows_the_theme` | ios | a touch screen has no hover |
| `toolbar::an_item_shown_later_keeps_its_dom_place` | ios | 3416: the simulator's tap or drag did not drive the page |
| `toolbar::escape_stays_put_once_focus_left_the_bar` | ios | 3416: the simulator's tap or drag did not drive the page |
| `toolbar::fields_join_the_arrow_order` | ios | 3416: the simulator's tap or drag did not drive the page |
| `toolbar::fields_join_the_arrow_order` | macos desktop | 3432: WKWebView: ArrowLeft at a caret's start keeps the field |
| `toolbar::shortcut_help_lists_the_chords` | ios | 3416: the simulator's tap or drag did not drive the page |
| `toolbar::tabbing_back_in_returns_to_the_last_item` | ios | 3416: flaky: a tap on a toolbar item sometimes leaves the focus on the body |
| `tooltip::a_long_press_on_a_link_trigger_opens_no_link_menu` | desktop | 1126: no touch input under Xvfb |
| `tooltip::a_long_press_on_a_link_trigger_opens_no_link_menu` | native | 3237: the native driver reads no script |
| `tooltip::a_long_press_opens_it_on_touch` | desktop | 1126: no touch input under Xvfb |
| `tooltip::a_short_touch_cancels_the_open` | desktop | 1126: no touch input under Xvfb |
| `tooltip::a_short_touch_cancels_the_open` | native | 2838: the native driver holds no timers, the cancel is unobservable |
| `tooltip::escape_closes_it_under_the_pointer` | ios | a touch screen has no hover |
| `tooltip::escape_closes_it_under_the_pointer` | web | 1760: escape_closes_it_under_the_pointer_and_on_focus covers it |
| `tooltip::hover_opens_it_below_the_trigger_and_leaving_closes_it` | android | 996: no hover on touch; a long press opens it instead |
| `tooltip::hover_opens_it_below_the_trigger_and_leaving_closes_it` | ios | a touch screen has no hover |
| `tooltip::hover_opens_it_below_the_trigger_and_leaving_closes_it` | web | 1760: hover_shows_the_bubble_and_the_pointer_can_cross_the_gap places and leaves it |
| `tooltip::tab_focus_opens_it` | web | 1760: tab_focus_shows_the_bubble_and_leaving_hides_it covers it |
| `tooltip::the_pointer_can_rest_on_the_bubble` | android | 996: no hover on touch; a long press opens it instead |
| `tooltip::the_pointer_can_rest_on_the_bubble` | ios | a touch screen has no hover |
| `tooltip::the_pointer_can_rest_on_the_bubble` | web | 1760: hover_shows_the_bubble_and_the_pointer_can_cross_the_gap rests on it |
| `tour::android_back_ends_a_tour` | all but android | 1275: no Back key off Android |
| `tour::arrow_right_on_the_last_step_does_not_finish_the_tour` | ios | 3416: focus stays on the body after a Tab or a tap |
| `tour::arrow_right_on_the_last_step_does_not_finish_the_tour` | macos desktop | upstream: WebKit does not focus a clicked button |
| `tour::done_on_the_last_step_finishes` | ios | 3416: focus stays on the body after a Tab or a tap |
| `tour::done_on_the_last_step_finishes` | macos desktop | upstream: WebKit does not focus a clicked button |
| `tour::escape_ends_the_tour_and_returns_focus` | ios | 3416: focus stays on the body after a Tab or a tap |
| `tour::escape_ends_the_tour_and_returns_focus` | macos desktop | upstream: WebKit does not focus a clicked button |
| `tour::escape_in_a_target_closes_its_open_select_first` | android | 959: a WebView handle queries no focusables to bridge by |
| `tour::escape_in_a_target_closes_its_open_select_first` | desktop | 959: a WebView handle queries no focusables to bridge by |
| `tour::escape_in_a_target_closes_its_open_select_first` | ios | 3416: the simulator's tap or drag did not drive the page |
| `tour::tab_follows_positive_tabindex_in_an_interactive_target` | android | 959: a WebView handle queries no focusables to bridge by |
| `tour::tab_follows_positive_tabindex_in_an_interactive_target` | desktop | 959: a WebView handle queries no focusables to bridge by |
| `tour::tab_follows_positive_tabindex_in_an_interactive_target` | ios | 3416: focus stays on the body after a Tab or a tap |
| `tour::tab_moves_between_the_card_and_an_interactive_target` | android | 959: a WebView handle queries no focusables to bridge by |
| `tour::tab_moves_between_the_card_and_an_interactive_target` | desktop | 959: a WebView handle queries no focusables to bridge by |
| `tour::tab_moves_between_the_card_and_an_interactive_target` | ios | 3416: the simulator's tap or drag did not drive the page |
| `tour::tab_skips_a_hidden_edge_of_an_interactive_target` | android | 959: a WebView handle queries no focusables to bridge by |
| `tour::tab_skips_a_hidden_edge_of_an_interactive_target` | desktop | 959: a WebView handle queries no focusables to bridge by |
| `tour::tab_skips_a_hidden_edge_of_an_interactive_target` | ios | 3416: focus stays on the body after a Tab or a tap |
| `tour::tab_stays_in_the_card` | android | 959: Tab leaves a trap for the body on the WebView |
| `tour::tab_stays_in_the_card` | desktop | 959: Shift+Tab from the first control leaves a trap on the WebView |
| `tour::tab_stays_in_the_card` | ios | 3416: focus stays on the body after a Tab or a tap |
| `transition::it_animates_a_callers_from_state` | macos desktop | 3432: no transition runs on the runner (Reduce Motion?) |
| `transition::it_animates_in_on_mount_without_open` | macos desktop | 3432: no transition runs on the runner (Reduce Motion?) |
| `tree::a_vanished_rows_button_hands_focus_to_the_next_row` | native | 3406: Blitz hands the vanished button's focus to a container div, parked with native |
| `use_back::android_back_steps_down_below_a_menu` | all but android | 2003: no Back key off Android |
| `use_back::android_back_steps_down_twice_without_a_tap` | all but android | 3322: no Back key off Android |
| `use_back::android_back_walks_the_routes` | all but android | 3384: no Back key off Android |
| `use_hotkeys::mod_k_counts_each_press_once` | ios | 3416: the simulator's tap or drag did not drive the page |
| `use_hotkeys::unmounting_one_hotkey_instance_keeps_the_other_listening` | ios | 3416: the simulator's tap or drag did not drive the page |
| `use_hotkeys::unmounting_the_component_stops_the_hotkey` | ios | 3416: the simulator's tap or drag did not drive the page |
| `use_intersection::a_target_is_in_the_viewport_only_while_scrolled_to` | ios | 3416: the simulator's tap or drag did not drive the page |
| `use_intersection::once_keeps_the_first_sighting` | ios | 3416: the simulator's tap or drag did not drive the page |
| `use_local_storage::the_local_area_outlives_a_relaunch` | android | no relaunch in the Android driver; its store is the files directory |
| `use_local_storage::the_local_area_outlives_a_relaunch` | desktop | a launch gets a fresh E2E_STORAGE_DIR, so a relaunch keeps nothing |
| `use_local_storage::the_local_area_outlives_a_relaunch` | native | Blitz has no relaunch in the driver |
| `use_long_press::a_held_touch_fires_the_callback_once_and_swallows_the_click` | desktop | 1126: no touch input under Xvfb |
| `use_long_press::a_short_touch_still_clicks` | desktop | 1126: no touch input under Xvfb |
| `use_swipe::a_mouse_drag_is_never_a_swipe` | android | adb drags with a finger |
| `use_swipe::a_mouse_drag_is_never_a_swipe` | ios | 3416: the simulator's tap or drag did not drive the page |
| `use_swipe::a_touch_swipe_reports_its_direction_past_the_distance` | desktop | 1126: no touch input under Xvfb |
| `use_swipe::an_edge_swipe_opens_from_the_right_under_rtl` | desktop | 1126: no touch input under Xvfb |
| `use_swipe::an_edge_swipe_opens_only_from_the_band_past_the_back_zone` | desktop | 1126: no touch input under Xvfb |
| `use_swipe::an_edge_swipe_reopens_a_drawer_five_times` | desktop | 1126: no touch input under Xvfb |
| `use_swipe::an_edge_swipe_wins_over_a_scrolled_inner_scroller` | desktop | 1126: no touch input under Xvfb |
| `use_timers::typing_settles_a_debounced_value_and_callback` | ios | 3416: the simulator's tap or drag did not drive the page |
| `video::every_audio_source_failing_shows_the_error` | native | Blitz plays no media and shows the fallback |
| `video::every_audio_source_failing_shows_the_error` | web | audio::every_source_failing_shows_the_error is the web's arm |
| `video::every_video_source_failing_shows_the_error` | native | Blitz plays no media and shows the fallback |
| `video::every_video_source_failing_shows_the_error` | web | video::every_source_failing_shows_the_error is the web's arm |
| `video::fullscreen::the_fullscreen_button_toggles` | ios | 3416: the simulator's tap or drag did not drive the page |
| `video::fullscreen::the_fullscreen_button_toggles` | macos desktop | upstream: Escape does not leave a WKWebView fullscreen |
| `viewport_meta::a_small_field_keeps_the_page_scale` | native | Blitz has no visual viewport |
| `viewport_meta::a_small_field_keeps_the_page_scale` | web | the web build carries its app's own index.html |
| `viewport_meta::the_viewport_covers_the_notch` | native | Blitz has no viewport meta |
| `viewport_meta::the_viewport_covers_the_notch` | web | the web build carries its app's own index.html |
