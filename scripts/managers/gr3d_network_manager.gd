class_name GR3DNet

const LOG_PREFIX = "[GR3DNet]: "
const MAX_CLIENTS = 6

static func start_server(port: int, tree: Node):
	var peer = ENetMultiplayerPeer.new()
	peer.create_server(port, MAX_CLIENTS)
	tree.multiplayer.multiplayer_peer = peer

static func connect_to_server(ip: String, port: int, tree: Node):
	var peer = ENetMultiplayerPeer.new()
	peer.create_client(ip, port)
	tree.multiplayer.multiplayer_peer = peer

static func start_sync(tree: Node):
	if tree.multiplayer.is_server():
		print(LOG_PREFIX, "start_sync starting..")
		await tree.get_tree().create_timer(2.0).timeout
		GR3DSync.start()

static func reset(tree: Node):
	tree.multiplayer.multiplayer_peer = null
	#GR3DSync.stop()
	#GR3DSync.clear_peers()

static func on_ready(tree: Node):
	GR3DNet.connect_console(tree)
	#GR3DSync.connect("sync_error", func(): GR3DNet.reset(tree))
	tree.multiplayer.connect("peer_connected", GR3DSync.add_peer)
	tree.multiplayer.connect("peer_disconnected", GR3DSync.remove_peer)

# Connects print statement functions to connection events
static func connect_console(tree: Node):
	GR3DSync.connect("peer_added", func(id: int): print(LOG_PREFIX, "peer_added: ", id))
	GR3DSync.connect("peer_removed", func(id: int): print(LOG_PREFIX, "peer_removed: ", id))
	#GR3DSync.connect("sync_lost", func(): print(LOG_PREFIX, "sync_lost"))
	#GR3DSync.connect("sync_regained", func(): print(LOG_PREFIX, "sync_regained"))
	#GR3DSync.connect("sync_error", func(msg: String): push_error(LOG_PREFIX, "sync_error: ", msg))
	tree.multiplayer.connect("peer_connected", func(id: int): print(LOG_PREFIX, "peer_connected: ", id))
	tree.multiplayer.connect("peer_disconnected", func(id: int): print(LOG_PREFIX, "peer_disconnected: ", id))
	tree.multiplayer.connect("connected_to_server", func(): print(LOG_PREFIX, "connected_to_server as: ", tree.multiplayer.get_unique_id()))
	tree.multiplayer.connect("connection_failed", func(): print(LOG_PREFIX, "connection_failed"))
	tree.multiplayer.connect("server_disconnected", func(): print(LOG_PREFIX, "server_disconnected"))
