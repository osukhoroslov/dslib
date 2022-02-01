from PySide2 import QtCore, QtWidgets, QtGui
import math
import typing as t
from static.const import STATIC_PATH, NodePlotRule


class DisplayedNode(QtWidgets.QGraphicsItemGroup):
    ICON_PATH = f"{STATIC_PATH}/pics/node.png"
    CROSS_PATH = f"{STATIC_PATH}/pics/cross.png"

    def __init__(self, node_id: str, size: t.Tuple[int, int], parent: QtWidgets.QWidget):
        QtWidgets.QGraphicsItemGroup.__init__(self, parent)
        self._id = node_id
        
        self._pic = QtWidgets.QGraphicsPixmapItem(QtGui.QPixmap(self.ICON_PATH).scaled(size[0], size[1]))
        self.addToGroup(self._pic)

        cross_pixmap = QtGui.QPixmap(self.CROSS_PATH).scaledToWidth(size[0] * 1.2)
        self._cross = QtWidgets.QGraphicsPixmapItem(cross_pixmap)
        self._cross.hide()
        self._cross.setPos(size[0] // 2 - cross_pixmap.width() // 2, size[1] // 2 - cross_pixmap.height() // 2)
        self.addToGroup(self._cross)

        self._text = QtWidgets.QGraphicsTextItem(node_id)
        self._text.setFont(QtGui.QFont("Times", 12, QtGui.QFont.Bold))
        self._text.setPos(size[0] // 2 - self._text.boundingRect().width() // 2, size[1])
        self.addToGroup(self._text)

        self.setFlag(QtWidgets.QGraphicsItem.ItemIsMovable)
        self.setFlag(QtWidgets.QGraphicsItem.ItemIsSelectable)
        # self.setFlag(QtWidgets.QGraphicsItem.ItemSendsGeometryChanges)
        self._connections_counter = 0  # for lines, that represent events (disables movement)
        
    def update_conn_counter(self, val: int):
        assert val in [-1, 1]  # TODO: remove this?
        was_zero = (self._connections_counter == 0)
        self._connections_counter += val
        if self._connections_counter == 0:
            self.setFlags(self.flags() | QtWidgets.QGraphicsItem.ItemIsMovable)
        elif was_zero:
            self.setFlags(self.flags() & ~QtWidgets.QGraphicsItem.ItemIsMovable)
    
    def show_cross(self):
        self._cross.show()
    
    def hide_cross(self):
        self._cross.hide()
    
    # def hoverEnterEvent(self, event: QtWidgets.QGraphicsSceneHoverEvent) -> None:
    #     return super().hoverEnterEvent(event)

    # def hoverLeaveEvent(self, event: QtWidgets.QGraphicsSceneHoverEvent) -> None:
    #     return super().hoverLeaveEvent(event)


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
        # self.setSceneRect(0, 0, rect.width(), rect.height())
        self._scene.setParent(self)
        # self._scene.setBackgroundBrush(QtCore.Qt.green)
        self.setScene(self._scene)
        
        self._node_ids = node_ids
        self.displayed_nodes: t.Dict[str, DisplayedNode] = {}
        self._node_icon_size: t.Optional[t.Tuple[int, int]] = None
        
    def on_startup(self):
        self._scene.clear()
        self.displayed_nodes.clear()
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
            self.displayed_nodes[node_id] = DisplayedNode(node_id, node_size, None)
            self._scene.addItem(self.displayed_nodes[node_id])
            self.displayed_nodes[node_id].setPos(x, y)
            # self._displayed_nodes[node_id].show()
    
    def mousePressEvent(self, event: QtGui.QMouseEvent) -> None:
        self.setDragMode(QtWidgets.QGraphicsView.ScrollHandDrag)
        return super().mousePressEvent(event)
    
    def mouseReleaseEvent(self, event: QtGui.QMouseEvent) -> None:
        self.setDragMode(QtWidgets.QGraphicsView.NoDrag)
        return super().mouseReleaseEvent(event)

    def wheelEvent(self, event: QtGui.QWheelEvent) -> None:
        if event.modifiers() & QtCore.Qt.ControlModifier:
            if event.delta() > 0:
                self.scale(2, 2)
            else:
                self.scale(0.5, 0.5)
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
            center = (max_x // 2, max_y // 2 - node_size[1] // 2)
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
