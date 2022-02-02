import typing as t
import datetime

from PySide2 import QtCore, QtWidgets

from static.const import MsgBoxColors


class MessageBox(QtWidgets.QWidget):
    def __init__(self, parent: t.Optional[QtWidgets.QWidget] = None) -> None:
        QtWidgets.QWidget.__init__(self, parent)
        
        self._main_layout = QtWidgets.QVBoxLayout(self)
        
        # self._name_lbl = QtWidgets.QLabel(self)
        # self._name_lbl.setText('MESSAGE BOX')
        # self._main_layout.addWidget(self._name_lbl,  alignment=QtCore.Qt.AlignCenter)
        # self._main_layout.addWidget(QtWidgets.QFrame(self, frameShape=QtWidgets.QFrame.HLine))

        self._level_lbl = QtWidgets.QLabel(self)
        self._main_layout.addWidget(self._level_lbl, alignment=QtCore.Qt.AlignCenter)

        self._text_lbl = QtWidgets.QLabel(self)
        self._main_layout.addWidget(self._text_lbl, alignment=QtCore.Qt.AlignCenter)

        self._ts_lbl = QtWidgets.QLabel(self)
        self._main_layout.addWidget(self._ts_lbl, alignment=QtCore.Qt.AlignCenter)

        self.setLayout(self._main_layout)

    def info(self, text: str, custom_level: str = 'INFO'):
        self._level_lbl.setText(custom_level)
        self._level_lbl.setStyleSheet(f'color: {MsgBoxColors.GREEN}')
        self._text_lbl.setText(text)
        self._ts_lbl.setText(datetime.datetime.now().strftime('%H:%M:%S'))
    
    def warning(self, text: str, custom_level: str = 'WARNING'):
        self._level_lbl.setText(custom_level)
        self._level_lbl.setStyleSheet(f'color: {MsgBoxColors.YELLOW}')
        self._text_lbl.setText(text)
        self._ts_lbl.setText(datetime.datetime.now().strftime('%H:%M:%S'))

    def error(self, text: str, custom_level: str = 'ERROR'):
        self._level_lbl.setText(custom_level)
        self._level_lbl.setStyleSheet(f'color: {MsgBoxColors.RED}')
        self._text_lbl.setText(text)
        self._ts_lbl.setText(datetime.datetime.now().strftime('%H:%M:%S'))

    def clear(self):
        self._level_lbl.clear()
        self._text_lbl.clear()
        self._ts_lbl.clear()
