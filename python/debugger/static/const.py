from enum import Enum

DEBUGGER_PATH = '../../dslib/python/debugger'
STATIC_PATH = f'{DEBUGGER_PATH}/static'
NEXT_STEP_DELAY = 200
EVENT_STEP_TO_ANIM_STEP_RATIO = 5
ENVELOPE_STEPS_COUNT = 20


class NodePlotRule(int, Enum):
    CIRCLE = 0
    ROW = 1
    COLUMN = 2

class EventType(str, Enum):
    MESSAGE_SEND = "MessageSend"
    MESSAGE_RECEIVE = "MessageReceive"
    LOCAL_MESSAGE_SEND = "LocalMessageSend"
    LOCAL_MESSAGE_RECEIVE = "LocalMessageReceive"
    MESSAGE_DROPPED = "MessageDropped"
    MESSAGE_DISCARDED = "MessageDiscarded"
    TIMER_SET = "TimerSet"
    TIMER_FIRED = "TimerFired"
    NODE_RECOVERED = "NodeRecovered"
    NODE_RESTARTED = "NodeRestarted"
    NODE_CRASHED = "NodeCrashed"
    NODE_CONNECTED = "NodeConnected"
    NODE_DISCONNECTED = "NodeDisconnected"
    LINK_ENABLED = "LinkEnabled"
    LINK_DISABLED = "LinkDisabled"
    NETWORK_PARTITION = "NetworkPartition"


class MsgBoxColors(str, Enum):
    GREEN = "green"
    YELLOW = "#F98800"
    RED = "red"

class OnMouseEventColor(str, Enum):
    MESSAGE_SEND = "green"
    MESSAGE_RECEIVE = "violet"
    LOCAL_MESSAGE_RECEIVE = "purple"
    TIMER_FIRED = "red"

