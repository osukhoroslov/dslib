import typing as t

from PySide2 import QtCore, QtWidgets, QtGui

class ButtonSet(QtWidgets.QWidget):
    def __init__(self, parent: t.Optional[QtWidgets.QWidget] = None) -> None:
        QtWidgets.QWidget.__init__(self, parent)

        self._button_set_layout = QtWidgets.QHBoxLayout(self)

        self.next_button = QtWidgets.QPushButton('Next (D) -->', self)
        self.prev_button = QtWidgets.QPushButton('<-- Prev (A)', self)
        self.run_button = QtWidgets.QPushButton('Run (S)', self)
        self.stop_button = QtWidgets.QPushButton('Stop (S)', self)

        self.prev_button.setStyleSheet("padding-left: 10px; padding-right: 10px;")
        self.next_button.setStyleSheet("padding-left: 10px; padding-right: 10px;")
        self.run_button.setStyleSheet("padding-left: 10px; padding-right: 10px;")
        self.stop_button.setStyleSheet("padding-left: 10px; padding-right: 10px;")
        
        self.stop_button.setEnabled(False)

        self.prev_button.setShortcut(QtGui.QKeySequence(QtCore.Qt.Key_A))
        self.next_button.setShortcut(QtGui.QKeySequence(QtCore.Qt.Key_D))

        self._button_set_layout.addWidget(self.prev_button, alignment=QtCore.Qt.AlignLeft)
        self._button_set_layout.addWidget(self.stop_button, alignment=QtCore.Qt.AlignCenter)
        self._button_set_layout.addWidget(self.run_button, alignment=QtCore.Qt.AlignCenter)
        self._button_set_layout.addWidget(self.next_button, alignment=QtCore.Qt.AlignRight)

        self.setLayout(self._button_set_layout)
