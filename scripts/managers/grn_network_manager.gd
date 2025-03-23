class_name GRNNet

const LOG_PREFIX = "[GRN]: "
const MAX_CLIENTS = 6
const LOGS_DIR = "user://grn_logs"
static var logging_enabled := true

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
		SyncManager.start()

static func reset(tree: Node):
	tree.multiplayer.multiplayer_peer = null
	SyncManager.stop()
	SyncManager.clear_peers()

static func on_ready(tree: Node):
	GRNNet.connect_console(tree)
	SyncManager.connect("sync_started", func(): start_logging(tree))
	SyncManager.connect("sync_stopped", SyncManager.stop_logging)
	SyncManager.connect("sync_error", func(): GRNNet.reset(tree))
	tree.multiplayer.connect("peer_connected", SyncManager.add_peer)
	tree.multiplayer.connect("peer_disconnected", SyncManager.remove_peer)

static func start_logging(tree: Node):
	if !logging_enabled: return
	var dir = DirAccess.make_dir_recursive_absolute(LOGS_DIR)
	var datetime = Time.get_datetime_dict_from_system()
	var file_name = "%04d%02d%02d-%02d%02d%02d-peer-%d.log" % [
		datetime["year"],
		datetime["month"],
		datetime["day"],
		datetime["hour"],
		datetime["minute"],
		datetime["second"],
		tree.multiplayer.get_unique_id()
	]
	SyncManager.start_logging(LOGS_DIR + "/" + file_name)

# Connects print statement functions to connection events
static func connect_console(tree: Node):
	SyncManager.connect("peer_added", func(id: int): print(LOG_PREFIX, "peer_added: ", id))
	SyncManager.connect("peer_removed", func(id: int): print(LOG_PREFIX, "peer_removed: ", id))
	SyncManager.connect("sync_lost", func(): print(LOG_PREFIX, "sync_lost"))
	SyncManager.connect("sync_regained", func(): print(LOG_PREFIX, "sync_regained"))
	SyncManager.connect("sync_error", func(msg: String): push_error(LOG_PREFIX, "sync_error: ", msg))
	tree.multiplayer.connect("peer_connected", func(id: int): print(LOG_PREFIX, "peer_connected: ", id))
	tree.multiplayer.connect("peer_disconnected", func(id: int): print(LOG_PREFIX, "peer_disconnected: ", id))
	tree.multiplayer.connect("connected_to_server", func(): print(LOG_PREFIX, "connected_to_server as: ", tree.multiplayer.get_unique_id()))
	tree.multiplayer.connect("connection_failed", func(): print(LOG_PREFIX, "connection_failed"))
	tree.multiplayer.connect("server_disconnected", func(): print(LOG_PREFIX, "server_disconnected"))
