import typing as t
from PySide2 import QtWidgets

# TODO: test this class
class JsonViewer(QtWidgets.QTreeWidget):
    def __init__(self, json_value, parent: t.Optional[QtWidgets.QWidget] = None) -> None:
        QtWidgets.QTreeWidget.__init__(self, parent)
        self.setHeaderLabel("Message data")
        self.header().hide()

        self.header().setStretchLastSection(False)
        self.header().setSectionResizeMode(QtWidgets.QHeaderView.ResizeToContents)

        self.fill_item(self.invisibleRootItem(), json_value)


    def fill_item(self, item, value):
        if value is None:
            return
        elif isinstance(value, dict):
            for key, val in sorted(value.items()):
                if isinstance(val, (int, float, str)):
                    self.new_item(item, f'{key}: [{type(val).__name__}] = {val}')
                else:
                    self.new_item(item, f'{key}: [{type(val).__name__}]', val)
        elif isinstance(value, (list, tuple)):
            for val in value:
                text = (str(val) if not isinstance(val, (dict, list, tuple))
                        else f'[{type(val).__name__}]')
                self.new_item(item, text, val)
        else:
            self.new_item(item, str(value))

    def new_item(self, parent, text, value=None):
        child = QtWidgets.QTreeWidgetItem([text])
        # child.setFlags(child.flags() | QtCore.Qt.ItemIsEditable)
        self.fill_item(child, value)
        parent.addChild(child)
