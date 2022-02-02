import typing as t

from PySide2 import QtCore, QtWidgets, QtGui
from PySide2.QtCharts import QtCharts
from util import Test, SessionData

class TestsView(QtWidgets.QWidget):
    def __init__(self, status: str, parent: t.Optional[QtWidgets.QWidget] = None) -> None:
        QtWidgets.QWidget.__init__(self, parent)

        self._scroll_area = QtWidgets.QScrollArea(self, widgetResizable=True)
        self._scroll_wgt = QtWidgets.QWidget(self)
        self._scroll_layout = QtWidgets.QVBoxLayout(self._scroll_wgt)
        self._scroll_area.setWidget(self._scroll_wgt)

        self._main_lbl = QtWidgets.QLabel(status, self)
        self._main_lbl.setFont(QtGui.QFont("Times", 16, QtGui.QFont.Bold))

        self._main_layout = QtWidgets.QVBoxLayout(self)
        self._main_layout.addWidget(self._main_lbl, alignment=QtCore.Qt.AlignCenter)
        self._main_layout.addWidget(self._scroll_area)

        self.setLayout(self._main_layout)
        self._tests: t.Dict[str, QtWidgets.QPushButton] = {}
    
    def add_test(self, test: Test, callback):
        if test.name in self._tests:
            return
        self._tests[test.name] = QtWidgets.QPushButton(test.name, self._scroll_wgt)
        self._tests[test.name].clicked.connect(callback)
        self._scroll_layout.addWidget(self._tests[test.name])

class StartupPage(QtWidgets.QWidget):
    def __init__(self, session_data: SessionData, button_callbacks: t.List, parent: t.Optional[QtWidgets.QWidget] = None) -> None:
        QtWidgets.QWidget.__init__(self, parent)

        self._main_layout = QtWidgets.QVBoxLayout()
        self._session_data = session_data
        # PIE CHART
        self._pie = QtCharts.QPieSeries(self)
        # self._pie.hovered.connect(self.show_slice)
        test_status_counters = {status: 0 for status in [Test.Status.PASSED, Test.Status.FAILED]}
        status_to_color = {
            Test.Status.PASSED: QtGui.QColor('green'),
            Test.Status.FAILED: QtGui.QColor('red')
        }
        for test in session_data.tests.values():
            test_status_counters[test.status] += 1
        for status, counter in test_status_counters.items():
            self._pie.append(f'{status}({counter})', counter).setBrush(status_to_color[status])
        self._pie.setLabelsVisible(True)

        self._chart = QtCharts.QChart()
        self._chart.addSeries(self._pie)
        self._chart.createDefaultAxes()
        self._chart.setAnimationOptions(QtCharts.QChart.SeriesAnimations)
        self._chart.setTitleFont(QtGui.QFont("Times", 24, QtGui.QFont.Bold))
        self._chart.setTitle("Test session info")
 
        self._chart.legend().setVisible(True)
        self._chart.legend().setAlignment(QtCore.Qt.AlignBottom)
 
        self._chartview = QtCharts.QChartView(self._chart)
        self._chartview.setRenderHint(QtGui.QPainter.Antialiasing)

        # TEST LIST
        self._tests_area = QtWidgets.QWidget(self)
        self._tests_layout = QtWidgets.QHBoxLayout(self._tests_area)
        self._tests_wgts = {
            Test.Status.PASSED: TestsView(Test.Status.PASSED, self),
            Test.Status.FAILED: TestsView(Test.Status.FAILED, self)
        }
        for i, test in enumerate(session_data.tests.values()):
            self._tests_wgts[test.status].add_test(test, button_callbacks[i])
        for tests_wgt in self._tests_wgts.values():
            self._tests_layout.addWidget(tests_wgt)
        self._tests_area.setLayout(self._tests_layout)

        # TODO: splitter?)
        self._main_layout.addWidget(self._chartview, 2)
        self._main_layout.addWidget(self._tests_area, 1)
        self.setLayout(self._main_layout)
    
    # def show_slice(self, slice: QtCharts.QPieSlice, is_hovered: bool):
    #     slice.setLabelVisible(is_hovered)
