#!/usr/bin/env python3
import curses.ascii
import curses.textpad
import os
import sys

os.environ.setdefault('ESCDELAY', '25')

KEY_CTRL_A = curses.ascii.SOH    #   1
KEY_CTRL_D = curses.ascii.EOT    #   4
KEY_CTRL_E = curses.ascii.ENQ    #   5
KEY_ENTER = curses.ascii.NL      #  10
KEY_CTRL_X = curses.ascii.CAN    #  24
KEY_ESC = curses.ascii.ESC       #  27
KEY_SPACE = curses.ascii.SP      #  32
KEY_PLUS = 43
KEY_MINUS = 45
KEY_2 = 50
KEY_DOWN = curses.KEY_DOWN       # 258
KEY_UP = curses.KEY_UP           # 259
KEY_HOME = curses.KEY_HOME       # 262
KEY_DELETE = curses.KEY_DC       # 330
KEY_INSERT = curses.KEY_IC       # 331
KEY_PAGEDOWN = curses.KEY_NPAGE  # 338
KEY_PAGEUP = curses.KEY_PPAGE    # 339
KEY_END = curses.KEY_END         # 360

MAX_EDIT_SIZE = 60
class CancelEdit(Exception): pass

ACTIONS = {
    'p': 'pick',
    'r': 'reword',
    'e': 'edit',
    's': 'squash',
    'f': 'fixup',
    'x': 'exec',
    'd': 'drop',
}

INSTRUCTIONS = """
Set action for highlighted item:
  P: pick (use commit)
  R: reword (use commit, but edit the commit message)
  E: edit (use commit, but stop for amending)
  S: squash (use commit, but meld into previous commit)
  F: fixup (like "squash", but discard this commit's log message)
  X: exec (run command (the rest of the line) using shell)
  D: drop (remove commit)
SPACE: select/deselect highlighted item
UP/DOWN: move highlighter. If an item is selected, also move it
ENTER: edit highlighted item (then ENTER to confirm, ESC to cancel)
+/-: insert an item/remove highlighted item
CTRL-X: quit and proceed with rebase
ESC: cancel and quit
""".split("\n")
INSTRUCTIONS = [line for line in INSTRUCTIONS if line.strip()]

class VisualRebase:
    def __init__(self, stdscr, file):
        self.stdscr = stdscr
        self.file = file
        self.init()
        self.loop()

    def init(self):
        curses.curs_set(0)
        self.color_counter = 0
        self.ATTR_HIGHLIGHT = self.init_color(bg=curses.COLOR_RED) | curses.A_BOLD
        self.ATTR_SELECTED = self.init_color(bg=curses.COLOR_YELLOW) | curses.A_BOLD
        self.ATTR_EDIT = self.init_color(bg=curses.COLOR_BLUE)

        self.items = []
        with open(self.file) as f:
            for line in f:
                line = line.strip()
                if line == '' or line[0] == '#':
                    continue
                action, content = line.split(' ', maxsplit=1)
                for action_code, act in ACTIONS.items():
                    if action == act:
                        break
                else:
                    raise Exception("Unknown action: %r" % action)
                self.items.append((action_code, content))
        if len(self.items) == 0:
            self.cancel()

        self.first_displayed_item = 0
        self.highlighted_item = 0
        self.selected = False

        self.available_lines = curses.LINES - len(INSTRUCTIONS) - 1
        self.calculate_constraints()
        self.draw_all_items()

        for index, line in enumerate(INSTRUCTIONS, self.available_lines + 1):
            self.draw_line(index, line)

        self.stdscr.refresh()

    def init_color(self, fg=curses.COLOR_WHITE, bg=curses.COLOR_BLACK):
        self.color_counter += 1
        curses.init_pair(self.color_counter, fg, bg)
        return curses.color_pair(self.color_counter)

    def loop(self):
        while True:
            ch = self.stdscr.getch()
            if ch == KEY_UP:
                self.move_highlight(new_highlighted_item=self.highlighted_item - 1)
            elif ch == KEY_DOWN:
                self.move_highlight(new_highlighted_item=self.highlighted_item + 1)
            elif ch == KEY_HOME:
                self.move_highlight(new_highlighted_item=0)
            elif ch == KEY_END:
                self.move_highlight(new_highlighted_item=self.last_item)
            elif ch == KEY_PAGEUP:
                self.page_move_highlight(-1)
            elif ch == KEY_PAGEDOWN:
                self.page_move_highlight(+1)
            elif ch == KEY_SPACE:
                self.toggle_selection()
            elif ch == KEY_ENTER:
                self.edit_highlight()
            elif ch in [KEY_MINUS, KEY_DELETE]:
                self.remove_item()
            elif ch in [KEY_PLUS, KEY_INSERT]:
                self.insert_item()
            elif ch == KEY_2:
                self.duplicate_item()
            elif ch == KEY_CTRL_X:
                self.save_and_proceed()
            elif ch == KEY_ESC:
                self.cancel()
            else:
                char = chr(ch).lower()
                if char in ACTIONS.keys():
                    self.set_action(char)
            self.stdscr.refresh()

    def page_move_highlight(self, signal):
        delta = self.available_item_lines - 1
        signed_delta = signal * delta
        self.first_displayed_item += signed_delta
        new_highlighted_item = self.highlighted_item + signed_delta
        self.move_highlight(new_highlighted_item)

    def move_highlight(self, new_highlighted_item):
        new_highlighted_item = max(new_highlighted_item, 0)
        new_highlighted_item = min(new_highlighted_item, self.last_item)

        if new_highlighted_item != self.highlighted_item:
            old_highlighted_item = self.highlighted_item
            self.highlighted_item = new_highlighted_item
            if self.selected:
                self.swap_items(old_highlighted_item, new_highlighted_item)

            self.adjust_scrolling()
            self.draw_all_items()

    def swap_items(self, i, j):
        self.items[i], self.items[j] = self.items[j], self.items[i]

    def toggle_selection(self):
        self.selected = not self.selected
        self.draw_item(item=self.highlighted_item)

    def edit_highlight(self):
        item = self.highlighted_item
        linenum = item - self.first_displayed_item
        content = self.items[item][1][:MAX_EDIT_SIZE-1]

        self.highlighted_item = None
        self.draw_item(item=item)
        self.highlighted_item = item
        self.stdscr.refresh()

        y = linenum + self.first_linenum
        win = curses.newwin(1, MAX_EDIT_SIZE, y, 10)
        win.addstr(0, 0, content)
        win.bkgd(0, self.ATTR_EDIT)
        win.move(0, 0)

        txt = curses.textpad.Textbox(win, insert_mode=True)
        def translate(key):
            if key == KEY_HOME:
                return KEY_CTRL_A
            elif key == KEY_END:
                return KEY_CTRL_E
            elif key == KEY_DELETE:
                return KEY_CTRL_D
            elif key == KEY_ESC:
                raise CancelEdit()
            else:
                return key

        curses.curs_set(2)
        try:
            content = txt.edit(translate).strip()
        except CancelEdit:
            edited = False
        else:
            action = self.items[item][0]
            self.items[item] = (action, content)
            edited = True
        curses.curs_set(0)

        self.draw_item(item=item)
        return edited

    def remove_item(self):
        if not self.items:
            return
        self.items.pop(self.highlighted_item)
        if self.highlighted_item >= len(self.items):
            self.highlighted_item -= 1
        self.calculate_constraints()
        self.draw_all_items()

    def insert_item(self):
        self.items.insert(self.highlighted_item, ('p', ''))
        self.calculate_constraints()
        self.draw_all_items()
        if not self.edit_highlight():
            self.items.pop(self.highlighted_item)
            self.calculate_constraints()
            self.draw_all_items()

    def duplicate_item(self):
        self.items.insert(self.highlighted_item, self.items[self.highlighted_item])
        self.highlighted_item += 1
        self.calculate_constraints()
        self.draw_all_items()

    def save_and_proceed(self):
        self.save_and_quit(self.items)

    def cancel(self):
        self.save_and_quit(items=[])

    def save_and_quit(self, items):
        with open(self.file, 'w') as f:
            for action_code, commit in items:
                action = ACTIONS[action_code]
                line = '%s %s' % (action, commit)
                print(line, file=f)
        sys.exit(0)

    def set_action(self, action_code):
        commit = self.items[self.highlighted_item][1]
        self.items[self.highlighted_item] = (action_code, commit)
        self.draw_item(item=self.highlighted_item)

    @property
    def last_item(self):
        return len(self.items) - 1

    @property
    def last_displayed_item(self):
        return self.first_displayed_item + self.available_item_lines - 1

    @last_displayed_item.setter
    def last_displayed_item(self, value):
        self.first_displayed_item = value - self.available_item_lines + 1

    def calculate_constraints(self):
        self.available_item_lines = self.available_lines
        self.first_linenum = 0
        self.scrollable = len(self.items) > self.available_item_lines
        if self.scrollable:
            self.available_item_lines -= 2
            self.first_linenum = 1

        self.adjust_scrolling()

    def adjust_scrolling(self):
        scroll_down = self.last_displayed_item - self.last_item
        if scroll_down > 0:
            self.first_displayed_item -= min(self.first_displayed_item, scroll_down)

        if self.highlighted_item > self.last_displayed_item:
            self.last_displayed_item = self.highlighted_item

        if self.first_displayed_item > self.highlighted_item:
            self.first_displayed_item = self.highlighted_item
        elif self.first_displayed_item < 0:
            self.first_displayed_item = 0

    def draw_all_items(self):
        num_lines = min(len(self.items), self.available_item_lines)
        for linenum in range(num_lines):
            self.draw_item(linenum=linenum)

        if self.scrollable:
            items_before = self.first_displayed_item
            items_after = len(self.items) - self.last_displayed_item - 1

            y = 0
            self.draw_line(y, "↑ %d" % items_before)

            y = self.available_item_lines + 1
            self.draw_line(y, "↓ %d" % items_after)
        else:
            for linenum in range(num_lines, self.available_lines):
                self.draw_line(linenum, "")

    def draw_item(self, linenum=None, item=None):
        assert (linenum is None) != (item is None)
        if item is None:
            item = linenum + self.first_displayed_item
        else:
            linenum = item - self.first_displayed_item

        y = linenum + self.first_linenum

        action_code, commit = self.items[item]
        action = ACTIONS[action_code].ljust(6)

        prefix, suffix = '   ', '   '
        if item == self.highlighted_item:
            if self.selected:
                prefix, suffix = ' < ', ' > '
                attr = self.ATTR_SELECTED
            else:
                attr = self.ATTR_HIGHLIGHT
        else:
            attr = 0

        line = prefix + action + ' ' + commit + suffix
        self.draw_line(y, line, attr)

    def draw_line(self, y, line, *attrs):
        self.stdscr.addstr(y, 0, line, *attrs)
        self.stdscr.clrtoeol()

curses.wrapper(VisualRebase, file=sys.argv[1])
