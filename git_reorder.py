#!/usr/bin/env python3
import sys

FILE_ARG_DELIMITER = '--'

def wrapper_main(my_path, options):
    import subprocess

    editor_cmd = my_path + ' ' + FILE_ARG_DELIMITER
    cmd = [
        'git',
        '-c', 'sequence.editor=' + editor_cmd,
        'rebase',
        '-i',
    ] + options
    status = subprocess.call(cmd)
    sys.exit(status)

def editor_main(file):
    import curses
    import os
    os.environ.setdefault('ESCDELAY', '25')

    KEY_ENTER = 10
    KEY_ESC = 27
    KEY_SPACE = 32

    HIGHLIGHTED = 1
    SELECTED = 2

    ACTIONS = {
        'p': 'pick',
        'r': 'reword',
        'e': 'edit',
        's': 'squash',
        'f': 'fixup',
        'd': 'drop',
    }

    INSTRUCTIONS = """
Set action on highlighted commit:
  P - pick (use commit)
  R - reword (use commit, but edit the commit message)
  E - edit (use commit, but stop for amending)
  S - squash (use commit, but meld into previous commit)
  F - fixup (like "squash", but discard this commit's log message)
  D - drop (remove commit)
SPACE - select/deselect commit item highlighted
UP/DOWN - move highlighter. If a commit is selected, also move it
ENTER - confirm and quit
ESC - cancel and quit
    """.split("\n")
    INSTRUCTIONS = [line for line in INSTRUCTIONS if line.strip()]

    class Reorder:
        def __init__(self, file):
            self.file = file

        def main(self, stdscr):
            self.stdscr = stdscr
            self.init()
            self.loop()

        def init(self):
            curses.curs_set(0)
            curses.init_pair(HIGHLIGHTED, curses.COLOR_WHITE, curses.COLOR_RED)
            curses.init_pair(SELECTED, curses.COLOR_WHITE, curses.COLOR_YELLOW)

            self.items = []
            with open(self.file) as f:
                for line in f:
                    line = line.strip()
                    if line != '' and line[0] != '#' and line != 'noop':
                        assert line.startswith('pick ')
                        commit = line[5:].strip()
                        self.items.append(('p', commit))
            if len(self.items) == 0:
                self.cancel_and_quit()

            self.first_displayed_item = 0
            self.highlighted_item = 0
            self.selected = False

            available_lines = curses.LINES - len(INSTRUCTIONS) - 1
            self.available_lines = available_lines
            self.first_linenum = 0
            self.scrollable = len(self.items) > self.available_lines
            if self.scrollable:
                self.available_lines -= 2
                self.first_linenum = 1

            self.draw_all_items()

            for index, line in enumerate(INSTRUCTIONS, available_lines + 1):
                self.draw_line(index, line)

            self.stdscr.refresh()

        def loop(self):
            while True:
                ch = self.stdscr.getch()
                if ch == curses.KEY_UP:
                    self.move_highlight(-1)
                elif ch == curses.KEY_DOWN:
                    self.move_highlight(+1)
                elif ch == KEY_SPACE:
                    self.selected = not self.selected
                    self.draw_item(item=self.highlighted_item)
                elif ch == KEY_ENTER:
                    self.save_and_quit()
                elif ch == KEY_ESC:
                    self.cancel_and_quit()
                else:
                    char = chr(ch).lower()
                    if char in ACTIONS.keys():
                        self.set_action(char)
                self.stdscr.refresh()

        def move_highlight(self, delta):
            new_highlighted_item = self.highlighted_item + delta
            if new_highlighted_item >= 0 and new_highlighted_item < len(self.items):
                old_highlighted_item = self.highlighted_item
                self.highlighted_item = new_highlighted_item
                if self.selected:
                    self.swap_items(old_highlighted_item, new_highlighted_item)

                if self.highlighted_item < self.first_displayed_item:
                    assert self.scrollable
                    self.first_displayed_item -= 1
                    self.draw_all_items()
                elif self.highlighted_item > self.last_displayed_item:
                    assert self.scrollable
                    self.first_displayed_item += 1
                    self.draw_all_items()
                else:
                    self.draw_item(item=old_highlighted_item)
                    self.draw_item(item=new_highlighted_item)

        def swap_items(self, i, j):
            self.items[i], self.items[j] = self.items[j], self.items[i]

        def save_and_quit(self):
            with open(self.file, 'w') as f:
                for action_code, commit in self.items:
                    action = ACTIONS[action_code]
                    line = '%s %s' % (action, commit)
                    print(line, file=f)
            sys.exit(0)

        def cancel_and_quit(self):
            with open(self.file, 'w') as f:
                print('', file=f)
            sys.exit(0)

        def set_action(self, action_code):
            _, commit = self.items[self.highlighted_item]
            self.items[self.highlighted_item] = (action_code, commit)
            self.draw_item(item=self.highlighted_item)

        @property
        def last_displayed_item(self):
            return self.first_displayed_item + self.available_lines - 1

        def draw_all_items(self):
            num_lines = min(len(self.items), self.available_lines)
            for linenum in range(num_lines):
                self.draw_item(linenum=linenum)

            if self.scrollable:
                items_before = self.first_displayed_item
                items_after = len(self.items) - self.last_displayed_item - 1

                y = 0
                self.draw_line(y, "↑ %d" % items_before)

                y = self.available_lines + 1
                self.draw_line(y, "↓ %d" % items_after)

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
                    attr = curses.A_BOLD | curses.color_pair(SELECTED)
                else:
                    attr = curses.A_BOLD | curses.color_pair(HIGHLIGHTED)
            else:
                attr = 0

            line = prefix + action + ' ' + commit + suffix
            self.draw_line(y, line, attr)

        def draw_line(self, y, line, *attrs):
            self.stdscr.addstr(y, 0, line, *attrs)
            self.stdscr.clrtoeol()

    reorder = Reorder(file)
    curses.wrapper(reorder.main)

options = sys.argv[1:]
if len(options) == 2 and options[0] == FILE_ARG_DELIMITER:
    editor_main(file=options[1])
else:
    INCOMPATIBLE_OPTIONS = {
        '--continue', '--abort', '--quit', '--skip', '--edit-todo',
        '--ignore-whitespace', '--whitespace', '--committer-date-is-author-date',
        '--ignore-date', '--signoff', '-i', '--interactive', '-x', '--exec'
    }
    for opt in options:
        if opt in INCOMPATIBLE_OPTIONS:
            sys.exit("Rebase option %r cannot be used" % opt)
        elif opt.startswith('--whitespace='):
            sys.exit("Rebase option %r cannot be used" % '--whitespace')
    wrapper_main(sys.argv[0], options)
