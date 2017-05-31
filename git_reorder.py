#!/usr/bin/env python3
import sys
import os.path

def wrapper_main(num_commits):
    import subprocess
    my_path = sys.argv[0]

    if num_commits <= 0:
        base = '--root'
    else:
        base = 'HEAD~%d' % num_commits

    cmd = [
        'git',
        '-c', 'core.editor=' + my_path,
        'rebase',
        '-i',
        base,
    ]
    status = subprocess.call(cmd)
    sys.exit(status)

def editor_main(file):
    import curses
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

    class Reorder:
        def __init__(self, file):
            self.file = file

        def main(self, stdscr):
            self.stdscr = stdscr
            self.init()
            self.loop()

        def init(self):
            curses.curs_set(0)

            self.items = []
            with open(self.file) as f:
                for line in f:
                    line = line.strip()
                    if line != '' and line[0] != '#':
                        assert line.startswith('pick ')
                        commit = line[5:].strip()
                        self.items.append(('p', commit))

            self.highlighted = 0
            self.selected = False

            curses.init_pair(HIGHLIGHTED, curses.COLOR_WHITE, curses.COLOR_RED)
            curses.init_pair(SELECTED, curses.COLOR_WHITE, curses.COLOR_YELLOW)

            for index in range(len(self.items)):
                self.draw_line(index)
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
                    self.draw_line(self.highlighted)
                elif ch == KEY_ENTER:
                    self.save()
                    break
                elif ch == KEY_ESC:
                    self.cancel()
                    break
                else:
                    char = chr(ch).lower()
                    if char in ACTIONS.keys():
                        self.set_action(char)
                self.stdscr.refresh()

        def move_highlight(self, delta):
            new_highlight = self.highlighted + delta
            if new_highlight >= 0 and new_highlight < len(self.items):
                old_highlight = self.highlighted
                self.highlighted = new_highlight
                if self.selected:
                    self.items[old_highlight], self.items[new_highlight] = self.items[new_highlight], self.items[old_highlight]

                self.draw_line(old_highlight)
                self.draw_line(new_highlight)

        def save(self):
            with open(self.file, 'w') as f:
                for action_code, commit in self.items:
                    action = ACTIONS[action_code]
                    line = '%s %s' % (action, commit)
                    print(line, file=f)

        def cancel(self):
            with open(self.file, 'w') as f:
                print('', file=f)

        def set_action(self, action_code):
            _, commit = self.items[self.highlighted]
            self.items[self.highlighted] = (action_code, commit)
            self.draw_line(self.highlighted)

        def draw_line(self, index):
            action_code, commit = self.items[index]
            action = ACTIONS[action_code].ljust(6)

            prefix, suffix = '   ', '   '
            if index == self.highlighted:
                if self.selected:
                    prefix, suffix = ' < ', ' > '
                    attr = curses.A_BOLD | curses.color_pair(SELECTED)
                else:
                    attr = curses.A_BOLD | curses.color_pair(HIGHLIGHTED)
            else:
                attr = 0

            line = prefix + action + ' ' + commit + suffix
            self.stdscr.addstr(index, 0, line, attr)
            self.stdscr.clrtoeol()

    reorder = Reorder(file)
    curses.wrapper(reorder.main)

args = sys.argv[1:]
if len(args) == 1:
    arg = args[0]
    try:
        num_commits = int(arg)
    except ValueError:
        if os.path.isfile(arg):
            editor_main(file=arg)
        elif arg == '--root':
            wrapper_main(num_commits=-1)
        else:
            sys.exit("What should I do with %r?!" % arg)
    else:
        wrapper_main(num_commits)
elif len(args) == 0:
    sys.exit("What?!")
else:
    sys.exit("What should I do with %r?!" % args)
