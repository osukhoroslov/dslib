from PySide2 import QtCore, QtWidgets, QtGui
import math
import typing as t
from static.const import STATIC_PATH, NodePlotRule
from static.const import OnMouseEventColor


# declaration for usage in DisplayedNode
class CentralDisplay(QtWidgets.QGraphicsView):
    pass


class DisplayedNode(QtWidgets.QGraphicsItemGroup):
    ICON_PATH = f"{STATIC_PATH}/pics/node.png"
    CROSS_PATH = f"{STATIC_PATH}/pics/cross.png"
    LOCAL_USER_PATH = f"{STATIC_PATH}/pics/localuser.png"

    def __init__(self, node_id: str, size: t.Tuple[int, int], display: CentralDisplay, parent: QtWidgets.QWidget):
        QtWidgets.QGraphicsItemGroup.__init__(self, parent)
        self._id = node_id
        self._node_size = size
        self._display = display
        
        self._node = QtWidgets.QGraphicsPixmapItem(QtGui.QPixmap(self.ICON_PATH).scaled(size[0], size[1]))
        self.addToGroup(self._node)

        self._border = display.scene().addRect(
            0, 0, size[0], size[1], QtGui.QPen(QtGui.QColor(OnMouseEventColor.LOCAL_MESSAGE), 3)
        )
        self._border.hide()
        self.addToGroup(self._border)

        cross_pixmap = QtGui.QPixmap(self.CROSS_PATH).scaledToWidth(size[0] * 1.2)
        self._cross = QtWidgets.QGraphicsPixmapItem(cross_pixmap)
        self._cross.hide()
        self._cross.setPos(size[0] // 2 - cross_pixmap.width() // 2, size[1] // 2 - cross_pixmap.height() // 2)
        self.addToGroup(self._cross)

        self._text = QtWidgets.QGraphicsTextItem(node_id)
        self._text.setFont(QtGui.QFont("Times", 12, QtGui.QFont.Bold))
        self._text.setPos(size[0] // 2 - self._text.boundingRect().width() // 2, size[1])
        self.addToGroup(self._text)

        local_user_pixmap = QtGui.QPixmap(self.LOCAL_USER_PATH).scaledToWidth(size[0] * 0.7)
        self._local_user_size = (local_user_pixmap.width(), local_user_pixmap.height())
        self._local_user = QtWidgets.QGraphicsPixmapItem(local_user_pixmap)
        self._local_user.setPos(-local_user_pixmap.width(), -local_user_pixmap.height())
        self._local_user.hide()
        self.addToGroup(self._local_user)

        self.setFlag(QtWidgets.QGraphicsItem.ItemIsMovable)
        self.setFlag(QtWidgets.QGraphicsItem.ItemIsSelectable)
        # self.setFlag(QtWidgets.QGraphicsItem.ItemSendsGeometryChanges)

        self._connections_counter = 0  # for lines, that represent events (disables movement)
        self._border_show_counter = 0  # border can be really hidden only when this counter = 0
        self._local_user_show_counter = 0  # same as border counter for local user
        
    def update_conn_counter(self, val: int):
        assert val in [-1, 1]  # TODO: remove this?
        was_zero = (self._connections_counter == 0)
        self._connections_counter += val
        if self._connections_counter < 0:
            self._connections_counter = 0
            print('Something went wrong')
            return
        if self._connections_counter == 0:
            self.setFlags(self.flags() | QtWidgets.QGraphicsItem.ItemIsMovable)
        elif was_zero:
            self.setFlags(self.flags() & ~QtWidgets.QGraphicsItem.ItemIsMovable)
    
    def show_cross(self):
        self._cross.show()
    
    def hide_cross(self):
        self._cross.hide()
    
    def show_local_user(self):
        if self._local_user_show_counter == 0:
            self._local_user.show()
        self._local_user_show_counter += 1
        
    def hide_local_user(self):
        self._local_user_show_counter = (
            0 if self._local_user_show_counter == 0
            else self._local_user_show_counter - 1
        )
        if self._local_user_show_counter == 0:
            self._local_user.hide()
    
    def show_border(self):
        if self._border_show_counter == 0:
            self._border.show()
        self._border_show_counter += 1

    def hide_border(self):
        self._border_show_counter = (
            0 if self._border_show_counter == 0
            else self._border_show_counter - 1
        )
        if self._border_show_counter == 0:
            self._border.hide()

    # def mousePressEvent(self, event: QtWidgets.QGraphicsSceneMouseEvent) -> None:
    #     self.show_border()
    #     return super().mousePressEvent(event)

    # def mouseReleaseEvent(self, event: QtWidgets.QGraphicsSceneMouseEvent) -> None:
    #     self.hide_border()
    #     return super().mouseReleaseEvent(event)

# class MyGraphicsScene(QtWidgets.QGraphicsScene):
#     def __init__(self) -> None:
#         QtWidgets.QGraphicsScene.__init__(self)


class CentralDisplay(QtWidgets.QGraphicsView):
    def __init__(
        self, 
        node_ids: t.Set[str],
        parent: t.Optional[QtWidgets.QWidget] = None, 
    ) -> None:
        QtWidgets.QGraphicsView.__init__(self, parent)
        
        self.setTransformationAnchor(QtWidgets.QGraphicsView.AnchorUnderMouse)

        self._scene = QtWidgets.QGraphicsScene()
        # rect = self.contentsRect()
        # print(rect)
        self._scene.setParent(self)
        # self._scene.setBackgroundBrush(QtCore.Qt.green)
        self.setScene(self._scene)
        self.setSceneRect(-1000, -1000, 2000, 2000)
        
        self._node_ids = node_ids
        self.displayed_nodes: t.Dict[str, DisplayedNode] = {}
        self._node_icon_size: t.Optional[t.Tuple[int, int]] = None

    
    def clear(self):
        self._scene.clear()
        self.displayed_nodes.clear()

    def on_startup(self):
        self.clear()
        self.plot_nodes(self._node_ids)

    def plot_nodes(self, node_ids: t.Set[str], plot_rule: NodePlotRule = NodePlotRule.CIRCLE):
        points = self.calc_node_positions(plot_rule)
        node_size = self.get_node_icon_size()
        if self.is_ids_ints():
            node_ids = sorted(self._node_ids, key=int)
        else:
            node_ids = sorted(self._node_ids)
        for num, node_id in enumerate(node_ids):
            x, y = points[num]
            self.displayed_nodes[node_id] = DisplayedNode(node_id, node_size, self, None)
            self._scene.addItem(self.displayed_nodes[node_id])
            self.displayed_nodes[node_id].setPos(x, y)
        self.updateGeometry()
    
    def mousePressEvent(self, event: QtGui.QMouseEvent) -> None:
        self.setDragMode(QtWidgets.QGraphicsView.ScrollHandDrag)
        return super().mousePressEvent(event)
    
    def mouseReleaseEvent(self, event: QtGui.QMouseEvent) -> None:
        self.setDragMode(QtWidgets.QGraphicsView.NoDrag)
        return super().mouseReleaseEvent(event)

    def wheelEvent(self, event: QtGui.QWheelEvent) -> None:
        if event.modifiers() & QtCore.Qt.ControlModifier:
            if event.delta() > 0:
                self.scale(1.2, 1.2)
            else:
                self.scale(1/1.2, 1/1.2)
        else:
            super().wheelEvent(event)
        
    ##### HELPERS ###
    def calc_node_positions(self, plot_rule: NodePlotRule = NodePlotRule.CIRCLE) -> t.List[t.Tuple[int, int]]:
        # return circled coords
        max_x, max_y = (
            self.width(), self.height()
        )
        node_size = self.get_node_icon_size()
        if plot_rule == NodePlotRule.CIRCLE:
            radius, curr_angle = (
                min(
                    max_x // 2 - node_size[0], 
                    max_y // 2 - node_size[1]
                ), 
                math.pi / 2
            )
            angle_delta = 2 * math.pi / len(self._node_ids)
            center = (0 - node_size[0] // 2, 0 - node_size[1] // 2)
            resulting_points = []
            for i in range(len(self._node_ids)):
                resulting_points.append((
                    center[0] + math.cos(curr_angle) * radius,
                    center[1] - math.sin(curr_angle) * radius,
                ))
                curr_angle -= angle_delta
            return resulting_points
        if plot_rule == NodePlotRule.ROW:
            raise NotImplementedError()
        if plot_rule == NodePlotRule.COLUMN:
            raise NotImplementedError()
    
    # TODO: add @attribute
    def get_node_icon_size(self):
        if self._node_icon_size is None:
            pixmap = QtGui.QPixmap(DisplayedNode.ICON_PATH)
            pixmap = pixmap.scaledToHeight(self.height() // 8)
            self._node_icon_size = (
                pixmap.width(), pixmap.height()
            )
            return self._node_icon_size
        return self._node_icon_size
    
    def is_ids_ints(self):
        return all(
            node_id.isdigit() for node_id in self._node_ids
        )
