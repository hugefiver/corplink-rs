start() {
    XRAY_CONFIG_FILE="/etc/xray.json"
    ENABLE_PROXY=1

    while [[ $# -gt 0 ]]; do
        case "$1" in
        -X | --no-proxy)
            ENABLE_PROXY=0
            shift
            ;;
        -c | --xray-config)
            XRAY_CONFIG_FILE="$2"
            shift 2
            ;;
        -v | --verbose)
            log_level="info"
            case "$2" in
            warn | info | debug | trace)
                log_level="$2"
                shift 2
                ;;
            *)
                shift
                ;;
            esac

            export RUST_LOG="$log_level"
            ;;
        *)
            if [[ -z "$CONFIG_FILE" ]]; then
                CONFIG_FILE="$1"
            fi
            shift
            ;;
        esac
    done

    if [[ -z "$CONFIG_FILE" ]]; then
        CONFIG_FILE="/corplink.json"
    fi

    if [[ "$ENABLE_PROXY" -eq 1 ]]; then
        echo "[proxy] enable proxy: xray"
        /usr/local/bin/xray run -c "$XRAY_CONFIG_FILE" &
    fi

    echo "[corplink] start corplink-rs"
    /corplink-rs "$CONFIG_FILE"
}

start "$@"
