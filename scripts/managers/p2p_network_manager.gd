class_name P2PNet

const LOG_PREFIX = "[P2P]: "
const MAX_CLIENTS = 6

static func reset(tree: Node):
	tree.multiplayer.multiplayer_peer = null

static func host(port: int, tree: Node):
	var peer = ENetMultiplayerPeer.new()
	peer.create_server(port, MAX_CLIENTS)
	tree.multiplayer.multiplayer_peer = peer

static func join(ip: String, port: int, tree: Node):
	var peer = ENetMultiplayerPeer.new()
	peer.create_client(ip, port)
	tree.multiplayer.multiplayer_peer = peer

# Connects print statement functions to connection events
static func connect_console(tree: Node):
	tree.multiplayer.connect("peer_connected", func(id: int): print(LOG_PREFIX, "peer_connected: ", id))
	tree.multiplayer.connect("peer_disconnected", func(id: int): print(LOG_PREFIX, "peer_disconnected: ", id))
	tree.multiplayer.connect("connected_to_server", func(): print(LOG_PREFIX, "connected_to_server as: ", tree.multiplayer.get_unique_id()))
	tree.multiplayer.connect("connection_failed", func(): print(LOG_PREFIX, "connection_failed"))
	tree.multiplayer.connect("server_disconnected", func(): print(LOG_PREFIX, "server_disconnected"))
