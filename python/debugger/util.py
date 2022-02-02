import json
import typing as t
from dataclasses import dataclass
from static.const import EventType
from PySide2 import QtWidgets


@dataclass
class Event:
    EVENT_TYPES = [
        val for val in EventType
    ]
    type: str
    data: t.Dict[str, t.Any]
    idx: int

    @staticmethod
    def from_json(json_event: str, idx: int):
        parsed = json.loads(json_event)
        assert parsed['type'] in Event.EVENT_TYPES, f'Got unexpected event type: {parsed["type"]}'
        return Event(
            parsed['type'],
            parsed['data'],
            idx
        )
    
    def to_json(self, indent=None):
        return json.dumps({
            'type': self.type,
            'data': self.data
        }, indent=indent)
    
    def __str__(self):
        return self.to_json()


@dataclass
class Test:
    class Status:
        PASSED = "PASSED"
        FAILED = "FAILED"

    name: int
    events: t.List[Event]
    status: Status = None
    err: t.Optional[str] = None

    def to_json(self, indent=None):
        return json.dumps({
            'name': self.name,
            'events': [
                {'type': event.type, 'data': event.data} for event in self.events
            ],
            'status': self.status,
            'err': self.err
        }, indent=indent)

    def __str__(self):
        return self.to_json()


@dataclass
class SessionData:
    tests: t.Dict[str, Test]
    node_ids: t.Set[str]


@dataclass
class TestDebugData:
    test: Test
    event_idx: int = 0


class FramedGroup(QtWidgets.QFrame):
    def __init__(
            self, 
            widgets: t.Dict[str, QtWidgets.QWidget], 
            layout_cls: t.Union[QtWidgets.QHBoxLayout, QtWidgets.QVBoxLayout], 
            parent: QtWidgets.QWidget = None
    ):
        QtWidgets.QFrame.__init__(self, parent)

        self.setLayout(layout_cls(self))
        self.widgets: t.Dict[str, QtWidgets.QWidget] = {}
        for name, widget in widgets.items():
            self.widgets[name] = widget
            self.layout().addWidget(widget)
