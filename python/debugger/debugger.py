import sys
import typing as t

from PySide2 import QtCore, QtWidgets, QtGui

from nodedisplay import CentralDisplay
from right_menu import RightMenu
from util import Test, SessionData, TestDebugData, FramedGroup
from button_set import ButtonSet
from messagebox import MessageBox
from static.const import NEXT_STEP_DELAY
from logparser import LogParser
from startuppage import StartupPage

# TODO: enable running from file? (for docker users)
# Add error dialog?
# add clear button
# add run backwards

class VDebugger:  # remove class?
    def __init__(self) -> None:
        self._session_data: SessionData = None

    def main(self, file_path: str):
        parser = LogParser()
        parser.parse_log_file(file_path)
        self._session_data = SessionData(parser.tests, parser.node_ids)
        # self.print_tests()
        self.start_gui()
    
    ############ GUI ############
    def start_gui(self):
        app = QtWidgets.QApplication([])
        main_window = MainWindow(self._session_data)
        screen_size = app.primaryScreen().size()
        main_window.resize(screen_size.width() // 2, screen_size.height() // 2)
        main_window.showMaximized()
        
        main_window.setFixedSize(main_window.size())
        # main_window.show()
        print(f'Debugger exited with status: {app.exec_()}')


class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, session_data: SessionData) -> None:
        QtWidgets.QMainWindow.__init__(self)
        self._session_data = session_data
        self._curr_test_debug_data: t.Optional[TestDebugData] = None

        self._menu_bar = self.menuBar()
        self._menu_bar.addAction('Main', self.show_main_page).setShortcut("Ctrl+T")

        self._tests_menus: t.Dict[str, QtWidgets.QMenu] = {
            'main': self._menu_bar.addMenu("Tests"),
            Test.Status.PASSED: None,
            Test.Status.FAILED: None
        }
        self._show_test_error_act = self._menu_bar.addAction('Show test error', self.show_test_error)
        self._show_test_error_act.setVisible(False)
        self._menu_bar.addAction('Quit', self.close).setShortcut("Ctrl+W")

        # add submenus by status
        for status in [Test.Status.PASSED, Test.Status.FAILED]:
            self._tests_menus[status] = self._tests_menus['main'].addMenu(status)
            self._tests_menus[status].setStyleSheet("menu-scrollable: 1;")
            self._tests_menus[status].setMaximumHeight(400)
        
        # set actions
        self._tests_menu_callbacks = []
        for test in self._session_data.tests.values():
            self._tests_menu_callbacks.append(self.on_select_test_wrapper(test.name))
            self._tests_menus[test.status].addAction(
                test.name, self._tests_menu_callbacks[-1]
            )

        # central widget
        self._central_widget = QtWidgets.QWidget(self)
        self.setCentralWidget(self._central_widget)

        self._central_layout = QtWidgets.QStackedLayout(self._central_widget)

        self._startup_page = StartupPage(self._session_data, self._tests_menu_callbacks, self)
        self._central_layout.addWidget(self._startup_page)

        # main widget in central layout
        self._horizontal_splitter = QtWidgets.QSplitter(QtCore.Qt.Horizontal, self)
        # splits node display and button_set with msg box
        self._vertical_splitter = QtWidgets.QSplitter(QtCore.Qt.Vertical, self)

        self._message_box = MessageBox(self)
        self._display = CentralDisplay(self._session_data.node_ids, self)
        self._right_menu = RightMenu(self._display, self)
        self._button_set = ButtonSet(self)

        self._left_frame = FramedGroup(
            {
                'splitter': self._vertical_splitter
            },
            QtWidgets.QHBoxLayout,
            self
        )

        # nodes display
        self._display_frame = FramedGroup(
            {
                'display': self._display
            },
            QtWidgets.QHBoxLayout,
            self
        )

        # lower buttons set
        self._btn_and_msg_frame = FramedGroup(
            {
                'button_set': self._button_set,
                'vline': QtWidgets.QFrame(self, frameShape=QtWidgets.QFrame.VLine),
                'message_box': self._message_box
            }, 
            QtWidgets.QHBoxLayout,
            self
        )
        
        # events
        self._right_frame = FramedGroup(
            {
                'right_menu': self._right_menu
            }, 
            QtWidgets.QHBoxLayout,
            self
        )
        self._left_frame.setStyleSheet('margin: 0px')
        self._right_frame.setStyleSheet('margin: 0px')

        # connect buttons
        self._button_set.next_button.clicked.connect(self.next_step)
        self._button_set.prev_button.clicked.connect(self.prev_step)
        self._button_set.rerun_button.clicked.connect(self.rerun)
        self._button_set.run_button.clicked.connect(self.run)
        self._button_set.stop_button.clicked.connect(self.stop)
        
        # timer for running
        self._timer = QtCore.QTimer()
        self._timer.timeout.connect(self.next_step)

        # final settings
        self._vertical_splitter.addWidget(self._display_frame)
        self._vertical_splitter.addWidget(self._btn_and_msg_frame)
        # self._vertical_splitter.setStretchFactor(0, 6)
        # self._vertical_splitter.setStretchFactor(1, 1)
        self._vertical_splitter.setSizes([60000, 10000])  # hack to set ratio, TODO: set in app

        self._horizontal_splitter.addWidget(self._left_frame)
        self._horizontal_splitter.addWidget(self._right_frame)
        # self._horizontal_splitter.setStretchFactor(0, 6)
        # self._horizontal_splitter.setStretchFactor(1, 4)
        self._horizontal_splitter.setSizes([60000, 40000])  # hack to set ratio
        self._central_layout.addWidget(self._horizontal_splitter)

        self._central_widget.setLayout(self._central_layout)
        self._central_widget.showMaximized()
        self.on_startup()

    def on_startup(self):
        self.setWindowTitle("VDebugger")
        # show test if it is single
        # if len(self._session_data.tests) == 1:
        #     test = list(self._session_data.tests.values())[0]
        #     self._tests_menus[test.status].actions()[0].trigger()
        # TODO: ONE TEST RUN IS BUGGING
    
    def on_select_test_wrapper(self, test_name: str):
        def on_select_test():
            test = self._session_data.tests[test_name]
            self.show_test_page()
            if test.err is not None:
                self._show_test_error_act.setVisible(True)
                # TODO: doesn't work...
                # self.show_test_error()
            else:
                self._show_test_error_act.setVisible(False)
            if self._curr_test_debug_data and self._curr_test_debug_data.test.name == test_name:
                # to start from where we were
                return

            self._curr_test_debug_data = TestDebugData(test)
            self.setWindowTitle(f"VDebugger | TEST: {test.name} | {test.status}")

            self._message_box.clear()

            self._right_menu.clear_events()

            # self._display.showMaximized()
            self._display.on_startup()
        return on_select_test
    
    def rerun(self):
        if self._timer.isActive():
            self.stop()
        self._curr_test_debug_data.event_idx = 0
        self._right_menu.clear_events()
        # self._display.on_startup()
        self.run()

    def run(self):
        if not self.is_test_selected():
            self._message_box.warning('Test is not selected!')
            return
        # disable buttons
        self._tests_menus['main'].setEnabled(False)
        self._button_set.prev_button.setEnabled(False)
        self._button_set.next_button.setEnabled(False)
        self._button_set.run_button.setEnabled(False)
        self._button_set.stop_button.setEnabled(True)

        curr_idx = self._curr_test_debug_data.event_idx
        if curr_idx >= len(self._curr_test_debug_data.test.events):
            # RESTART
            self.rerun()
        
        self.next_step()
        self._timer.start(NEXT_STEP_DELAY)

    def stop(self):
        self._tests_menus['main'].setEnabled(True)
        self._button_set.prev_button.setEnabled(True)
        self._button_set.next_button.setEnabled(True)
        self._button_set.run_button.setEnabled(True)
        self._button_set.stop_button.setEnabled(False)
        self._timer.stop()
    
    def next_step(self):
        if not self.is_test_selected():
            self._message_box.warning('Test is not selected!')
            return
        event_idx = self._curr_test_debug_data.event_idx
        if event_idx >= len(self._curr_test_debug_data.test.events):
            # TODO: think what here
            if self._timer.isActive():
                self.stop()
            self._message_box.info(f'Last event is reached (#{self._curr_test_debug_data.event_idx})')
            return
        event = self._curr_test_debug_data.test.events[
            self._curr_test_debug_data.event_idx
        ]
        self._message_box.info(
            f'Event: #{self._curr_test_debug_data.event_idx + 1}/'
            f'{len(self._curr_test_debug_data.test.events)}'
        )
        self._curr_test_debug_data.event_idx += 1
        self._right_menu.next_event(event)

    def prev_step(self):
        if not self.is_test_selected():
            self._message_box.warning('Test is not selected!')
            return

    def is_test_selected(self):
        return self._curr_test_debug_data is not None

    def show_test_error(self):
        if not self.is_test_selected():
            self._message_box.warning('Test is not selected!')
            return
        self._message_box.error(self._curr_test_debug_data.test.err, custom_level='TEST ERROR')
    
    def show_main_page(self):
        self._central_layout.setCurrentIndex(0)
        self._show_test_error_act.setVisible(False)
    
    def show_test_page(self):
        self._central_layout.setCurrentIndex(1)
    
    def keyPressEvent(self, event):
        if event.key() in [QtCore.Qt.Key_Space, QtCore.Qt.Key_S]:
            if self._timer.isActive():
                self.stop()
            else:
                self.run()


class UserDebugger:
    def __init__(self) -> None:
        pass

user_debugger = UserDebugger()

def get_debugger() -> UserDebugger:
    return user_debugger

if __name__ == '__main__':
    vdeb = VDebugger()
    vdeb.main(sys.argv[1])
