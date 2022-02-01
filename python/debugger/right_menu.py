from PySide2 import QtCore, QtWidgets, QtGui
import typing as t
from static.const import EventType, OnMouseEventColor
from static.const import STATIC_PATH, EVENT_STEP_TO_ANIM_STEP_RATIO, NEXT_STEP_DELAY, ENVELOPE_STEPS_COUNT
from util import Event
from nodedisplay import CentralDisplay
import hashlib
import json
from jsonviewer import JsonViewer

DISPLAYED_EVENT_GRID = (
    (1, 0, 1, 1),
    (0, 1, 3, 3)
)


class DisplayedEvent(QtWidgets.QWidget):
    def __init__(self, event: Event, display: CentralDisplay, parent: t.Optional[QtWidgets.QWidget] = None) -> None:
        QtWidgets.QWidget.__init__(self, parent)
        self._event = event
        self._display = display
        self._main_layout = QtWidgets.QGridLayout(self)
        self.setLayout(self._main_layout)


class DisplayedMsgSend(DisplayedEvent):
    def __init__(self, event: Event, display: CentralDisplay, parent: t.Optional[QtWidgets.QWidget] = None) -> None:
        DisplayedEvent.__init__(self, event, display, parent)
        caption = (
            f'{event.data["ts"]:.3f} | {event.data["src"]} --> {event.data["dst"]} | {event.data["msg"]["type"]}'
        )
        self._main_lbl = QtWidgets.QLabel(self)
        self._main_lbl.setAlignment(QtCore.Qt.AlignCenter)
        self._main_lbl.setText(caption)
        self._main_layout.addWidget(self._main_lbl, *DISPLAYED_EVENT_GRID[0])

        self._msg_viewer = JsonViewer(event.data['msg']['data'], self)
        self._main_layout.addWidget(self._msg_viewer, *DISPLAYED_EVENT_GRID[1])

        self._is_selected = False
        self._line: t.Optional[QtWidgets.QGraphicsLineItem] = None
        self._color: str = OnMouseEventColor.MESSAGE_SEND

        node_icon_size = self._display.get_node_icon_size()
        pixmap = QtGui.QPixmap(QtGui.QImage(f'{STATIC_PATH}/pics/envelope.png')).scaledToWidth(node_icon_size[0] // 2)
        self._envelope_size = (pixmap.width(), pixmap.height())
        self._envelope = QtWidgets.QGraphicsPixmapItem(pixmap)
        self._envelope_positions = self._calc_envelope_positions()
        self._envelope_pos_idx = 0
        self._envelope.setPos(
            self._envelope_positions[0][0],
            self._envelope_positions[0][1]
        )
        
        self._timer = QtCore.QTimer()
        self._timer.timeout.connect(self.advance_envelope)
    
    def mouseReleaseEvent(self, ev: QtGui.QMouseEvent) -> None:
        if not self._is_selected:
            self._main_lbl.setStyleSheet(f'background-color: {self._color};')
            self.draw_line()
        else:
            self._main_lbl.setStyleSheet('')
            self.remove_line()
        self._is_selected = not self._is_selected
        return super().mouseReleaseEvent(ev)
    
    def enterEvent(self, event: QtCore.QEvent) -> None:
        if not self._is_selected:
            self._main_lbl.setStyleSheet(f'background-color: {self._color};')
        return super().enterEvent(event)
    
    def leaveEvent(self, event: QtCore.QEvent) -> None:
        if not self._is_selected:
            self._main_lbl.setStyleSheet('')
        return super().leaveEvent(event)
    
    def draw_line(self):
        if self._line is not None:
            print(f"Line already drawn {self._event.data['src']} --> {self._event.data['dst']}")  # TODO: remove this
            return
        src_node, dst_node = (
            self._display.displayed_nodes[self._event.data['src']],
            self._display.displayed_nodes[self._event.data['dst']]
        )
        src_node.update_conn_counter(1)
        dst_node.update_conn_counter(1)

        node_icon_size = self._display.get_node_icon_size()
        src_x, src_y = (
            src_node.scenePos().x() + node_icon_size[0] // 2,
            src_node.scenePos().y() + node_icon_size[1] // 2
        )
        dst_x, dst_y = (
            dst_node.scenePos().x() + node_icon_size[0] // 2,
            dst_node.scenePos().y() + node_icon_size[1] // 2
        )
        self._line = self._display.scene().addLine(
            src_x, src_y, dst_x, dst_y, QtGui.QPen(QtGui.QColor(self._color), 3)
        )
        self._display.scene().addItem(self._envelope)
        self._timer.start(NEXT_STEP_DELAY / EVENT_STEP_TO_ANIM_STEP_RATIO)

    
    def remove_line(self):
        if self._line is None:
            print(f"Line already removed {self._event.data['src']} --> {self._event.data['dst']}")  # TODO: remove this
            return
        self._timer.stop()
        src_node, dst_node = (
            self._display.displayed_nodes[self._event.data['src']],
            self._display.displayed_nodes[self._event.data['dst']]
        )
        src_node.update_conn_counter(-1)
        dst_node.update_conn_counter(-1)

        self._display.scene().removeItem(self._line)
        self._display.scene().removeItem(self._envelope)
        self._line = None
    
    def advance_envelope(self):
        self._envelope.setPos(
            self._envelope_positions[self._envelope_pos_idx][0],
            self._envelope_positions[self._envelope_pos_idx][1]
        )
        self._envelope_pos_idx = (self._envelope_pos_idx + 1) % ENVELOPE_STEPS_COUNT
    
    def _calc_envelope_positions(self):
        src_node, dst_node = (
            self._display.displayed_nodes[self._event.data['src']],
            self._display.displayed_nodes[self._event.data['dst']]
        )
        node_icon_size = self._display.get_node_icon_size()
        src_x, src_y = (
            src_node.scenePos().x() + node_icon_size[0] // 2,
            src_node.scenePos().y() + node_icon_size[1] // 2
        )
        dst_x, dst_y = (
            dst_node.scenePos().x() + node_icon_size[0] // 2,
            dst_node.scenePos().y() + node_icon_size[1] // 2
        )
        step_x = (dst_x - src_x) / ENVELOPE_STEPS_COUNT
        step_y = (dst_y - src_y) / ENVELOPE_STEPS_COUNT
        return [
            (
                src_x + step_x * i - self._envelope_size[0] // 2, 
                src_y + step_y * i - self._envelope_size[1] // 2
            )
            for i in range(ENVELOPE_STEPS_COUNT)
        ]


class DisplayedMsgRcv(DisplayedEvent):
    def __init__(self, event: Event, display: CentralDisplay, parent: t.Optional[QtWidgets.QWidget] = None) -> None:
        DisplayedEvent.__init__(self, event, display, parent)
        caption = (
            f'{event.data["ts"]:.3f} | {event.data["dst"]} <-- {event.data["src"]} | {event.data["msg"]["type"]}'
        )
        self._main_lbl = QtWidgets.QLabel(self)
        self._main_lbl.setAlignment(QtCore.Qt.AlignCenter)
        self._main_lbl.setText(caption)
        self._main_layout.addWidget(self._main_lbl, *DISPLAYED_EVENT_GRID[0])

        self._msg_viewer = JsonViewer(event.data['msg']['data'], self)
        self._main_layout.addWidget(self._msg_viewer, *DISPLAYED_EVENT_GRID[1])

        self._is_selected = False
        self._line: t.Optional[QtWidgets.QGraphicsLineItem] = None
        self._color: str = OnMouseEventColor.MESSAGE_RECEIVE
    
    def mouseReleaseEvent(self, ev: QtGui.QMouseEvent) -> None:
        if not self._is_selected:
            self._main_lbl.setStyleSheet(f'background-color: {self._color};')
            self.draw_line()
        else:
            self._main_lbl.setStyleSheet('')
            self.remove_line()
        self._is_selected = not self._is_selected
        return super().mouseReleaseEvent(ev)

    def enterEvent(self, event: QtCore.QEvent) -> None:
        if not self._is_selected:
            self._main_lbl.setStyleSheet(f'background-color: {self._color};')
        return super().enterEvent(event)
    
    def leaveEvent(self, event: QtCore.QEvent) -> None:
        if not self._is_selected:
            self._main_lbl.setStyleSheet('')
        return super().leaveEvent(event)

    def draw_line(self):
        src_node, dst_node = (
            self._display.displayed_nodes[self._event.data['src']],
            self._display.displayed_nodes[self._event.data['dst']]
        )
        src_node, dst_node = (
            self._display.displayed_nodes[self._event.data['src']],
            self._display.displayed_nodes[self._event.data['dst']]
        )
        src_node.update_conn_counter(1)
        dst_node.update_conn_counter(1)

        node_icon_size = self._display.get_node_icon_size()
        src_x, src_y = (
            src_node.scenePos().x() + node_icon_size[0] // 2,
            src_node.scenePos().y() + node_icon_size[1] // 2
        )
        dst_x, dst_y = (
            dst_node.scenePos().x() + node_icon_size[0] // 2,
            dst_node.scenePos().y() + node_icon_size[1] // 2
        )
        self._line = self._display.scene().addLine(
            src_x, src_y, dst_x, dst_y, QtGui.QPen(QtGui.QColor(self._color), 3)
        )
    
    def remove_line(self):
        src_node, dst_node = (
            self._display.displayed_nodes[self._event.data['src']],
            self._display.displayed_nodes[self._event.data['dst']]
        )
        src_node.update_conn_counter(-1)
        dst_node.update_conn_counter(-1)

        self._display.scene().removeItem(self._line)
        self._line = None


class RightMenu(QtWidgets.QWidget):
    def __init__(self, display: CentralDisplay, parent: t.Optional[QtWidgets.QWidget] = None) -> None:
        QtWidgets.QWidget.__init__(self, parent)
        self._main_layout = QtWidgets.QVBoxLayout(self)
        self._events_scroll = QtWidgets.QScrollArea(self, widgetResizable=True)
        self._events_scroll.setVerticalScrollBarPolicy(QtCore.Qt.ScrollBarAlwaysOn)
        self._events_scroll.setHorizontalScrollBarPolicy(QtCore.Qt.ScrollBarAlwaysOff)
        self._events_wgt = QtWidgets.QWidget(self)
        self._events_layout = QtWidgets.QVBoxLayout(self._events_wgt)
        self._events_layout.addStretch(1)
        self._events_wgt.setLayout(self._events_layout)
        self._events_scroll.setWidget(self._events_wgt)
        self._main_layout.addWidget(self._events_scroll)
        self.setLayout(self._main_layout)

        self._display = display

        # {src: {dst: {id: t.SetDisplayedMsgSend}}}
        self._pending_msgs: t.Dict[str, t.Dict[str, t.Dict[str, t.List[DisplayedMsgSend]]]] = {}

    
    def next_event(self, event: Event):
        if event.type == EventType.MESSAGE_SEND:
            key = hashlib.sha256(
                json.dumps(event.data['msg']).encode()
            ).hexdigest()
            src, dst = event.data['src'], event.data['dst']
            self._ensure_keys_exist(src, dst, key)
            self._pending_msgs[src][dst][key].append(DisplayedMsgSend(event, self._display, self._events_wgt))
            self._events_layout.addWidget(
                self._pending_msgs[src][dst][key][-1], alignment=QtCore.Qt.AlignTop
            )
            self._pending_msgs[src][dst][key][-1].draw_line()

        elif event.type == EventType.MESSAGE_RECEIVE:
            key = hashlib.sha256(
                json.dumps(event.data['msg']).encode()
            ).hexdigest()
            self._events_layout.addWidget(
                DisplayedMsgRcv(event, self._display, self._events_wgt), alignment=QtCore.Qt.AlignTop
            )
            self._pending_msgs[event.data['src']][event.data['dst']][key].pop().remove_line()
        elif event.type == EventType.NODE_CRASHED:
            self._display.displayed_nodes[event.data['node']].show_cross()
        elif event.type == EventType.NODE_RECOVERED:
            self._display.displayed_nodes[event.data['node']].hide_cross()
        
        self._events_scroll.verticalScrollBar().setValue(
            self._events_scroll.verticalScrollBar().maximum()
        )
    
    def _ensure_keys_exist(self, src: str, dst: str, key: str):
        if src not in self._pending_msgs:
            self._pending_msgs[src] = {dst:{key:[]}}
            return
        if dst not in self._pending_msgs[src]:
            self._pending_msgs[src][dst] = {key:[]}
            return
        if key not in self._pending_msgs[src][dst]:
            self._pending_msgs[src][dst][key] = []
            return
    
    def clear_events(self):
        self._pending_msgs.clear()
        layout = self._events_layout
        for i in reversed(range(layout.count())):
            item = layout.itemAt(i).widget()
            if item:
                item.deleteLater()

