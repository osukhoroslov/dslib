from __future__ import annotations
import abc
import json
from typing import Any, List, Dict, Tuple


class Message:
    def __init__(self, message_type: str, data: Dict[str, Any]):
        self._type = message_type
        self._data = data

    @property
    def type(self) -> str:
        return self._type

    def __getitem__(self, key: str) -> Any:
        return self._data[key]

    def __setitem__(self, key: str, value: Any):
        self._data[key] = value

    def remove(self, key: str):
        self._data.pop(key, None)

    def to_dict(self) -> Dict[str, Any]:
        return {'type': self._type, 'data': self._data}

    @staticmethod
    def from_json(message_type: str, json_str: str) -> Message:
        return Message(message_type, json.loads(json_str))

    @staticmethod
    def from_dict(message_dict: Dict[str, Any]):
        return Message(message_dict['type'], message_dict['data'])


class Context(object):
    def __init__(self, time: float):
        self._time = time
        self._sent_messages: List[Tuple[str, str, str]] = list()
        self._sent_local_messages: List[tuple[str, str]] = list()
        self._timer_actions: List[Tuple[str, float]] = list()

    def send(self, msg: Message, to: str):
        if not isinstance(to, str):
            raise TypeError('to argument has to be string, not {}'.format(type(to)))
        self._sent_messages.append((msg.type, json.dumps(msg._data), to))

    def send_local(self, msg: Message):
        self._sent_local_messages.append((msg.type, json.dumps(msg._data)))

    def set_timer(self, timer_id: str, delay: float):
        if not isinstance(timer_id, str):
            raise TypeError('timer_id argument has to be str, not {}'.format(type(timer_id)))
        if not isinstance(delay, (int, float)):
            raise TypeError('delay argument has to be int or float, not {}'.format(type(delay)))
        if delay < 0:
            raise ValueError('delay argument has to be non-negative')
        self._timer_actions.append((timer_id, delay))
    
    def cancel_timer(self, timer_id: str):
        if not isinstance(timer_id, str):
            raise TypeError('timer_id argument has to be str, not {}'.format(type(timer_id)))
        self._timer_actions.append((timer_id, -1))

    def time(self) -> float:
        return self._time


class State:
    def __init__(self, data: Dict[str, Any]):
        self._data = data

    @property
    def data(self) -> Dict[str, Any]:
        return self._data

    def __getitem__(self, key: str) -> Any:
        return self._data[key]

    def __setitem__(self, key: str, value: Any):
        self._data[key] = value

    @staticmethod
    def from_json(json_str: str) -> State:
        return State(json.loads(json_str))


class Node:
    @abc.abstractmethod
    def on_local_message(self, msg: Message, ctx: Context):
        pass

    @abc.abstractmethod
    def on_message(self, msg: Message, sender: str, ctx: Context):
        pass

    @abc.abstractmethod
    def on_timer(self, timer_id: str, ctx: Context):
        pass

    def get_json_state(self) -> str:
        return json.dumps(self.get_state().data)

    @abc.abstractmethod
    def get_state(self) -> State:
        pass

    def set_json_state(self, json_str: str):
        self.set_state(State.from_json(json_str))

    @abc.abstractmethod
    def set_state(self, state: State):
        pass
