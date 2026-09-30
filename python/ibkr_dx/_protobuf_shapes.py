"""The reference client's second encoding, present here as a routing.

Beside every text request that client publishes a ``*ProtoBuf`` twin which
serializes a request message of a protobuf encoding and sends it on a socket,
``useProtoBuf`` to ask which encoding to speak, and a ``*ProtoBuf`` stub on
its wrapper for every answer of that encoding. There is no socket between a
program and this client — the engine is in the same process — and no protobuf
encoder exists or is needed here: the family routes each message to the text
request carrying the same intent, and ``useProtoBuf`` answers False, so a
sample program written against that client takes its text path.

A call carries what its message states; a field the message does not state
falls to the text request's own default, and an argument the text request
requires but the message omits arrives as the encoding's own zero for it —
what a gateway reads off the wire for an unstated field. The Limits page of
the documentation states the family's behaviour beside the two config calls.

The wrapper's stubs exist to be found: a program's override calls ``super()``
into them, and the base answers as every other stub here answers — it does
nothing.
"""

import inspect

#: The classes a conversion fills, bound when `install` runs.
SURFACE = {}


def _stated(message):
    """The fields a message of that encoding states, as a dict.

    A real message answers `ListFields` with what was written; a stand-in —
    which is all a program without an encoder can build — carries its stated
    fields as attributes.
    """
    if isinstance(message, dict):
        return message
    listed = getattr(message, "ListFields", None)
    if listed is not None:
        return {field.name: value for field, value in listed()}
    return {name: value for name, value in vars(message).items()
            if not name.startswith("_") and callable(value) is False}


def _entries(value):
    """A map field, which that encoding carries as a list of entry messages,
    as the tag-and-value pairs the text surface carries."""
    return [SURFACE["TagValue"](entry.key, entry.value) for entry in value]


def _filled(kind, message, renames=None, converters=None):
    """One of this client's classes, carrying what a message states.

    Only the stated fields are written, so a field the message omits keeps
    the class's own default — its unset marker where it has one — instead of
    the encoding's zero standing in for a figure nobody gave.
    """
    made = kind()
    for name, value in _stated(message).items():
        if converters is not None and name in converters:
            value = converters[name](value)
        setattr(made, renames.get(name, name) if renames else name, value)
    return made


def _multiplier(value):
    """The message states a multiplier as a number; the text surface carries
    it as the digits the venue reads."""
    return str(int(value)) if value == int(value) else str(value)


def _plain(kind):
    def convert(message):
        return _filled(SURFACE[kind], message)
    return convert


def _contract(message):
    return _filled(
        SURFACE["Contract"], message,
        {"primaryExch": "primaryExchange"},
        {"multiplier": _multiplier,
         "deltaNeutralContract": _plain("DeltaNeutralContract"),
         "comboLegs": lambda legs: [_filled(SURFACE["ComboLeg"], leg)
                                    for leg in legs]})


def _order(message):
    return _filled(
        SURFACE["Order"], message, None,
        {"algoParams": _entries, "smartComboRoutingParams": _entries,
         "orderMiscOptions": _entries,
         "softDollarTier": lambda tier: _filled(SURFACE["SoftDollarTier"],
                                                tier, {"value": "val"})})


class _empty:
    """The zero for a required argument the message omits: an empty class
    instance, which is what an unstated message field carries on the wire."""

    def __init__(self, kind):
        self.kind = kind

    def __call__(self):
        return SURFACE[self.kind]()


#: An argument the text request requires, from a field the message states:
#: field -> (keyword, convert, zero). The zero is passed where the field is
#: absent; a callable zero is called then. An argument the text request
#: defaults is passed only where the message states its field.
def _call(twin, required=(), optional=()):
    def route(self, stated):
        kwargs = {}
        for field, kwarg, convert, zero in required:
            if field in stated:
                kwargs[kwarg] = convert(stated[field]) if convert else stated[field]
            else:
                kwargs[kwarg] = zero() if callable(zero) else zero
        for field, kwarg, convert in optional:
            if field in stated:
                kwargs[kwarg] = convert(stated[field]) if convert else stated[field]
        return getattr(self, twin)(**kwargs)
    return route


_R = ("reqId", "req_id", None, 0)
_C = ("contract", "contract", _contract, _empty("Contract"))

_SCANNER_FIELDS = ("scannerSubscriptionOptions", "scannerSubscriptionFilterOptions")


def _scanner_route(self, stated):
    """A scan: the subscription's scalars fill the class the text request
    takes, and its two option lists — which the text request carries as
    arguments of their own, not on the subscription — travel beside it."""
    kwargs = {"req_id": stated.get("reqId", 0)}
    subscription = SURFACE["ScannerSubscription"]()
    message = stated.get("scannerSubscription")
    if message is not None:
        fields = _stated(message)
        for kwarg, field in zip(("scanner_subscription_options",
                                 "scanner_subscription_filter_options"),
                                _SCANNER_FIELDS):
            if field in fields:
                kwargs[kwarg] = _entries(fields.pop(field))
        for name, value in fields.items():
            setattr(subscription, name, value)
    kwargs["subscription"] = subscription
    return self.req_scanner_subscription(**kwargs)


def _wsh_route(self, stated):
    """A watchlist event request: that encoding carries the filter's fields
    flat on the request, where the text request takes them on an object."""
    kwargs = {"req_id": stated.get("reqId", 0)}
    fields = {name: value for name, value in stated.items() if name != "reqId"}
    if fields:
        kwargs["wsh_event_data"] = _filled(SURFACE["WshEventData"], fields)
    return self.req_wsh_event_data(**kwargs)


def _nothing(twin):
    def route(self, stated):
        return getattr(self, twin)()
    return route


#: The client family: (this client's spelling, the reference's parameter
#: name, the route). Enumerated from the reference client's own definitions;
#: the two config calls answer from the Rust surface and are not here.
_CLIENT = (
    ("start_api_proto_buf", "startApiRequestProto", _nothing("start_api")),
    ("req_current_time_proto_buf", "currentTimeRequestProto", _nothing("req_current_time")),
    ("set_server_log_level_proto_buf", "setServerLogLevelRequestProto",
     _call("set_server_log_level", optional=(("logLevel", "log_level", None),))),
    ("req_market_data_proto_buf", "marketDataRequestProto",
     _call("req_mkt_data", (_R, _C),
           (("genericTickList", "generic_tick_list", None),
            ("snapshot", "snapshot", None),
            ("regulatorySnapshot", "regulatory_snapshot", None),
            ("marketDataOptions", "mkt_data_options", _entries)))),
    ("cancel_market_data_proto_buf", "cancelMarketDataProto",
     _call("cancel_mkt_data", (_R,))),
    ("req_market_data_type_proto_buf", "marketDataTypeRequestProto",
     _call("req_market_data_type", (("marketDataType", "market_data_type", None, 0),))),
    ("req_smart_components_proto_buf", "smartComponentsRequestProto",
     _call("req_smart_components", (_R, ("bboExchange", "bbo_exchange", None, "")))),
    ("req_market_rule_proto_buf", "marketRuleRequestProto",
     _call("req_market_rule", (("marketRuleId", "market_rule_id", None, 0),))),
    ("req_tick_by_tick_data_proto_buf", "tickByTickRequestProto",
     _call("req_tick_by_tick_data", (_R, _C, ("tickType", "tick_type", None, "")),
           (("numberOfTicks", "number_of_ticks", None),
            ("ignoreSize", "ignore_size", None)))),
    ("cancel_tick_by_tick_proto_buf", "cancelTickByTickProto",
     _call("cancel_tick_by_tick_data", (_R,))),
    ("calculate_implied_volatility_proto_buf", "calculateImpliedVolatilityRequestProto",
     _call("calculate_implied_volatility",
           (_R, _C, ("optionPrice", "option_price", None, 0.0),
            ("underPrice", "under_price", None, 0.0)),
           (("impliedVolatilityOptions", "implied_vol_options", _entries),))),
    ("cancel_calculate_implied_volatility_proto_buf", "cancelCalculateImpliedVolatilityProto",
     _call("cancel_calculate_implied_volatility", (_R,))),
    ("calculate_option_price_proto_buf", "calculateOptionPriceRequestProto",
     _call("calculate_option_price",
           (_R, _C, ("volatility", "volatility", None, 0.0),
            ("underPrice", "under_price", None, 0.0)),
           (("optionPriceOptions", "opt_prc_options", _entries),))),
    ("cancel_calculate_option_price_proto_buf", "cancelCalculateOptionPriceProto",
     _call("cancel_calculate_option_price", (_R,))),
    ("exercise_options_proto_buf", "exerciseOptionsRequestProto",
     _call("exercise_options",
           (("orderId", "req_id", None, 0), _C,
            ("exerciseAction", "exercise_action", None, 0),
            ("exerciseQuantity", "exercise_quantity", None, 0),
            ("account", "account", None, ""),
            ("override", "override", int, 0)),
           (("manualOrderTime", "manual_order_time", None),
            ("customerAccount", "customer_account", None),
            ("professionalCustomer", "professional_customer", None)))),
    # The message's attachedOrders has no place on the text request, which
    # carries a stop-loss or take-profit pair on the order's own fields; the
    # Limits page says so.
    ("place_order_proto_buf", "placeOrderRequestProto",
     _call("place_order", (("orderId", "order_id", None, 0), _C,
                           ("order", "order", _order, _empty("Order"))))),
    ("cancel_order_proto_buf", "cancelOrderRequestProto",
     _call("cancel_order", (("orderId", "order_id", None, 0),),
           (("orderCancel", "order_cancel", _plain("OrderCancel")),))),
    ("req_open_orders_proto_buf", "openOrdersRequestProto", _nothing("req_open_orders")),
    ("req_auto_open_orders_proto_buf", "autoOpenOrdersRequestProto",
     _call("req_auto_open_orders", (("autoBind", "b_auto_bind", None, False),))),
    ("req_all_open_orders_proto_buf", "allOpenOrdersRequestProto",
     _nothing("req_all_open_orders")),
    ("req_global_cancel_proto_buf", "globalCancelRequestProto",
     _call("req_global_cancel", optional=(("orderCancel", "order_cancel",
                                           _plain("OrderCancel")),))),
    ("req_ids_proto_buf", "idsRequestProto",
     _call("req_ids", optional=(("numIds", "num_ids", None),))),
    ("req_account_updates_proto_buf", "accountDataRequestProto",
     _call("req_account_updates", (("subscribe", "subscribe", None, False),),
           (("acctCode", "acct_code", None),))),
    ("req_account_summary_proto_buf", "accountSummaryRequestProto",
     _call("req_account_summary", (_R, ("group", "group_name", None, ""),
                                   ("tags", "tags", None, "")))),
    ("cancel_account_summary_proto_buf", "cancelAccountSummaryProto",
     _call("cancel_account_summary", (_R,))),
    ("req_positions_proto_buf", "positionsRequestProto", _nothing("req_positions")),
    ("cancel_positions_proto_buf", "cancelPositionsProto", _nothing("cancel_positions")),
    ("req_positions_multi_proto_buf", "positionsMultiRequestProto",
     _call("req_positions_multi", (_R, ("account", "account", None, ""),
                                   ("modelCode", "model_code", None, "")))),
    ("cancel_positions_multi_proto_buf", "cancelPositionsMultiProto",
     _call("cancel_positions_multi", (_R,))),
    ("req_account_updates_multi_proto_buf", "accountUpdatesMultiRequestProto",
     _call("req_account_updates_multi",
           (_R, ("account", "account", None, ""), ("modelCode", "model_code", None, "")),
           (("ledgerAndNLV", "ledger_and_nlv", None),))),
    ("cancel_account_updates_multi_proto_buf", "cancelAccountUpdatesMultiProto",
     _call("cancel_account_updates_multi", (_R,))),
    ("req_pnl_proto_buf", "pnlRequestProto",
     _call("req_pnl", (_R, ("account", "account", None, "")),
           (("modelCode", "model_code", None),))),
    ("cancel_pnl_proto_buf", "cancelPnLProto", _call("cancel_pnl", (_R,))),
    ("req_pnl_single_proto_buf", "pnlSingleRequestProto",
     _call("req_pnl_single", (_R, ("account", "account", None, ""),
                              ("modelCode", "model_code", None, ""),
                              ("conId", "con_id", None, 0)))),
    ("cancel_pnl_single_proto_buf", "cancelPnLSingleProto",
     _call("cancel_pnl_single", (_R,))),
    ("req_executions_proto_buf", "executionRequestProto",
     _call("req_executions", (_R,),
           (("executionFilter", "exec_filter", _plain("ExecutionFilter")),))),
    ("req_contract_data_proto_buf", "contractDataRequestProto",
     _call("req_contract_details", (_R, _C))),
    ("req_market_depth_exchanges_proto_buf", "marketDepthExchangesRequestProto",
     _nothing("req_mkt_depth_exchanges")),
    ("req_market_depth_proto_buf", "marketDepthRequestProto",
     _call("req_mkt_depth", (_R, _C),
           (("numRows", "num_rows", None),
            ("isSmartDepth", "is_smart_depth", None),
            ("marketDepthOptions", "mkt_depth_options", _entries)))),
    ("cancel_market_depth_proto_buf", "cancelMarketDepthProto",
     _call("cancel_mkt_depth", (_R,), (("isSmartDepth", "is_smart_depth", None),))),
    ("req_news_bulletins_proto_buf", "newsBulletinsRequestProto",
     _call("req_news_bulletins", optional=(("allMessages", "all_msgs", None),))),
    ("cancel_news_bulletins_proto_buf", "cancelNewsBulletinsProto",
     _nothing("cancel_news_bulletins")),
    ("req_managed_accts_proto_buf", "managedAccountsRequestProto",
     _nothing("req_managed_accts")),
    ("req_fa_proto_buf", "faRequestProto",
     _call("request_fa", (("faDataType", "fa_data_type", None, 0),))),
    ("replace_fa_proto_buf", "faReplaceProto",
     _call("replace_fa", (_R, ("faDataType", "fa_data_type", None, 0),
                          ("xml", "cxml", None, "")))),
    ("req_historical_data_proto_buf", "historicalDataRequestProto",
     _call("req_historical_data",
           (_R, _C, ("endDateTime", "end_date_time", None, ""),
            ("barSizeSetting", "bar_size_setting", None, ""),
            ("duration", "duration_str", None, ""),
            ("whatToShow", "what_to_show", None, ""),
            ("useRTH", "use_rth", int, 0)),
           (("formatDate", "format_date", None),
            ("keepUpToDate", "keep_up_to_date", None),
            ("chartOptions", "chart_options", _entries)))),
    ("cancel_historical_data_proto_buf", "cancelHistoricalDataProto",
     _call("cancel_historical_data", (_R,))),
    ("req_head_time_stamp_proto_buf", "headTimestampRequestProto",
     _call("req_head_time_stamp",
           (_R, _C, ("whatToShow", "what_to_show", None, ""),
            ("useRTH", "use_rth", int, 0)),
           (("formatDate", "format_date", None),))),
    ("cancel_head_time_stamp_proto_buf", "cancelHeadTimestampProto",
     _call("cancel_head_time_stamp", (_R,))),
    ("req_histogram_data_proto_buf", "histogramDataRequestProto",
     _call("req_histogram_data",
           (_R, _C, ("useRTH", "use_rth", None, False),
            ("timePeriod", "time_period", None, "")))),
    ("cancel_histogram_data_proto_buf", "cancelHistogramDataProto",
     _call("cancel_histogram_data", (_R,))),
    ("req_historical_ticks_proto_buf", "historicalTicksRequestProto",
     _call("req_historical_ticks", (_R, _C),
           (("startDateTime", "start_date_time", None),
            ("endDateTime", "end_date_time", None),
            ("numberOfTicks", "number_of_ticks", None),
            ("whatToShow", "what_to_show", None),
            ("useRTH", "use_rth", int),
            ("ignoreSize", "ignore_size", None),
            ("miscOptions", "misc_options", _entries)))),
    ("req_scanner_parameters_proto_buf", "scannerParametersRequestProto",
     _nothing("req_scanner_parameters")),
    ("req_scanner_subscription_proto_buf", "scannerSubscriptionRequestProto",
     _scanner_route),
    ("cancel_scanner_subscription_proto_buf", "cancelScannerSubscriptionProto",
     _call("cancel_scanner_subscription", (_R,))),
    ("req_real_time_bars_proto_buf", "realTimeBarsRequestProto",
     _call("req_real_time_bars", (_R, _C),
           (("barSize", "bar_size", None),
            ("whatToShow", "what_to_show", None),
            ("useRTH", "use_rth", int),
            ("realTimeBarsOptions", "real_time_bars_options", _entries)))),
    ("cancel_real_time_bars_proto_buf", "cancelRealTimeBarsProto",
     _call("cancel_real_time_bars", (_R,))),
    ("req_news_providers_proto_buf", "newsProvidersRequestProto",
     _nothing("req_news_providers")),
    ("req_news_article_proto_buf", "newsArticleRequestProto",
     _call("req_news_article",
           (_R, ("providerCode", "provider_code", None, ""),
            ("articleId", "article_id", None, "")),
           (("newsArticleOptions", "news_article_options", _entries),))),
    ("req_historical_news_proto_buf", "historicalNewsRequestProto",
     _call("req_historical_news",
           (_R, ("conId", "con_id", None, 0),
            ("providerCodes", "provider_codes", None, ""),
            ("startDateTime", "start_date_time", None, ""),
            ("endDateTime", "end_date_time", None, ""),
            ("totalResults", "total_results", None, 0)),
           (("historicalNewsOptions", "historical_news_options", _entries),))),
    ("query_display_groups_proto_buf", "queryDisplayGroupsRequestProto",
     _call("query_display_groups", (_R,))),
    ("subscribe_to_group_events_proto_buf", "subscribeToGroupEventsRequestProto",
     _call("subscribe_to_group_events", (_R, ("groupId", "group_id", None, 0)))),
    ("update_display_group_proto_buf", "updateDisplayGroupRequestProto",
     _call("update_display_group", (_R, ("contractInfo", "contract_info", None, "")))),
    ("unsubscribe_from_group_events_proto_buf", "unsubscribeFromGroupEventsRequestProto",
     _call("unsubscribe_from_group_events", (_R,))),
    ("verify_request_proto_buf", "verifyRequestProto",
     _call("verify_request", (("apiName", "api_name", None, ""),
                              ("apiVersion", "api_version", None, "")))),
    ("verify_message_proto_buf", "verifyMessageRequestProto",
     _call("verify_message", (("apiData", "api_data", None, ""),))),
    ("req_sec_def_opt_params_proto_buf", "secDefOptParamsRequestProto",
     _call("req_sec_def_opt_params",
           (_R, ("underlyingSymbol", "underlying_symbol", None, ""),
            ("futFopExchange", "fut_fop_exchange", None, ""),
            ("underlyingSecType", "underlying_sec_type", None, ""),
            ("underlyingConId", "underlying_con_id", None, 0)))),
    ("req_soft_dollar_tiers_proto_buf", "softDollarTiersRequestProto",
     _call("req_soft_dollar_tiers", (_R,))),
    ("req_family_codes_proto_buf", "familyCodesRequestProto", _nothing("req_family_codes")),
    ("req_matching_symbols_proto_buf", "matchingSymbolsRequestProto",
     _call("req_matching_symbols", (_R, ("pattern", "pattern", None, "")))),
    ("req_completed_orders_proto_buf", "completedOrdersRequestProto",
     _call("req_completed_orders", optional=(("apiOnly", "api_only", None),))),
    ("req_wsh_meta_data_proto_buf", "wshMetaDataRequestProto",
     _call("req_wsh_meta_data", (_R,))),
    ("cancel_wsh_meta_data_proto_buf", "cancelWshMetaDataProto",
     _call("cancel_wsh_meta_data", (_R,))),
    ("req_wsh_event_data_proto_buf", "wshEventDataRequestProto", _wsh_route),
    ("cancel_wsh_event_data_proto_buf", "cancelWshEventDataProto",
     _call("cancel_wsh_event_data", (_R,))),
    ("req_user_info_proto_buf", "userInfoRequestProto", _call("req_user_info", (_R,))),
    ("req_current_time_in_millis_proto_buf", "currentTimeInMillisRequestProto",
     _nothing("req_current_time_in_millis")),
    ("cancel_contract_data_proto_buf", "cancelContractDataProto",
     _call("cancel_contract_data", (_R,))),
    ("cancel_historical_ticks_proto_buf", "cancelHistoricalTicksProto",
     _call("cancel_historical_ticks", (_R,))),
)

#: The wrapper's stubs: (this client's spelling, the reference's parameter
#: name), enumerated from the reference wrapper's own definitions.
_WRAPPER = (
    ("order_status_proto_buf", "orderStatusProto"),
    ("open_order_proto_buf", "openOrderProto"),
    ("open_orders_end_proto_buf", "openOrdersEndProto"),
    ("error_proto_buf", "errorMessageProto"),
    ("execution_details_proto_buf", "executionDetailsProto"),
    ("execution_details_end_proto_buf", "executionDetailsProto"),
    ("completed_order_proto_buf", "completedOrderProto"),
    ("completed_orders_end_proto_buf", "completedOrdersEndProto"),
    ("order_bound_proto_buf", "orderBoundProto"),
    ("contract_data_proto_buf", "contractDataProto"),
    ("bond_contract_data_proto_buf", "contractDataProto"),
    ("contract_data_end_proto_buf", "contractDataEndProto"),
    ("tick_price_proto_buf", "tickPriceProto"),
    ("tick_size_proto_buf", "tickSizeProto"),
    ("tick_option_computation_proto_buf", "tickOptionComputationProto"),
    ("tick_generic_proto_buf", "tickGenericProto"),
    ("tick_string_proto_buf", "tickStringProto"),
    ("tick_snapshot_end_proto_buf", "tickSnapshotEndProto"),
    ("update_market_depth_proto_buf", "marketDepthProto"),
    ("update_market_depth_l2_proto_buf", "marketDepthL2Proto"),
    ("update_market_data_type_proto_buf", "marketDataTypeProto"),
    ("tick_req_params_proto_buf", "tickReqParamsProto"),
    ("update_account_value_proto_buf", "accountValueProto"),
    ("update_portfolio_proto_buf", "portfolioValueProto"),
    ("update_account_time_proto_buf", "accountUpdateTimeProto"),
    ("account_data_end_proto_buf", "accountDataEndProto"),
    ("managed_accounts_proto_buf", "managedAccountsProto"),
    ("position_proto_buf", "positionProto"),
    ("position_end_proto_buf", "positionEndProto"),
    ("account_summary_proto_buf", "accountSummaryProto"),
    ("account_summary_end_proto_buf", "accountSummaryEndProto"),
    ("position_multi_proto_buf", "positionMultiProto"),
    ("position_multi_end_proto_buf", "positionMultiEndProto"),
    ("account_update_multi_proto_buf", "accountUpdateMultiProto"),
    ("account_update_multi_end_proto_buf", "accountUpdateMultiEndProto"),
    ("historical_data_proto_buf", "historicalDataProto"),
    ("historical_data_update_proto_buf", "historicalDataUpdateProto"),
    ("historical_data_end_proto_buf", "historicalDataEndProto"),
    ("real_time_bar_tick_proto_buf", "realTimeBarTickProto"),
    ("head_timestamp_proto_buf", "headTimestampProto"),
    ("histogram_data_proto_buf", "histogramDataProto"),
    ("historical_ticks_proto_buf", "historicalTicksProto"),
    ("historical_ticks_bid_ask_proto_buf", "historicalTicksBidAskProto"),
    ("historical_ticks_last_proto_buf", "historicalTicksLastProto"),
    ("tick_by_tick_data_proto_buf", "tickByTickDataProto"),
    ("update_news_bulletin_proto_buf", "newsBulletinProto"),
    ("news_article_proto_buf", "newsArticleProto"),
    ("news_providers_proto_buf", "newsProvidersProto"),
    ("historical_news_proto_buf", "historicalNewsProto"),
    ("historical_news_end_proto_buf", "historicalNewsEndProto"),
    ("wsh_meta_data_proto_buf", "wshMetaDataProto"),
    ("wsh_event_data_proto_buf", "wshEventDataProto"),
    ("tick_news_proto_buf", "tickNewsProto"),
    ("scanner_parameters_proto_buf", "scannerParametersProto"),
    ("scanner_data_proto_buf", "scannerDataProto"),
    ("pnl_proto_buf", "pnlProto"),
    ("pnl_single_proto_buf", "pnlSingleProto"),
    ("receive_fa_proto_buf", "receiveFAProto"),
    ("replace_fa_end_proto_buf", "replaceFAEndProto"),
    ("commission_and_fees_report_proto_buf", "commissionAndFeesReportProto"),
    ("historical_schedule_proto_buf", "historicalScheduleProto"),
    ("reroute_market_data_request_proto_buf", "rerouteMarketDataRequestProto"),
    ("reroute_market_depth_request_proto_buf", "rerouteMarketDepthRequestProto"),
    ("sec_def_opt_parameter_proto_buf", "secDefOptParameterProto"),
    ("sec_def_opt_parameter_end_proto_buf", "secDefOptParameterEndProto"),
    ("soft_dollar_tiers_proto_buf", "softDollarTiersProto"),
    ("family_codes_proto_buf", "familyCodesProto"),
    ("symbol_samples_proto_buf", "symbolSamplesProto"),
    ("smart_components_proto_buf", "smartComponentsProto"),
    ("market_rule_proto_buf", "marketRuleProto"),
    ("user_info_proto_buf", "userInfoProto"),
    ("next_valid_id_proto_buf", "nextValidIdProto"),
    ("current_time_proto_buf", "currentTimeProto"),
    ("current_time_in_millis_proto_buf", "currentTimeInMillisProto"),
    ("verify_message_api_proto_buf", "verifyMessageApiProto"),
    ("verify_completed_proto_buf", "verifyCompletedProto"),
    ("display_group_list_proto_buf", "displayGroupListProto"),
    ("display_group_updated_proto_buf", "displayGroupUpdatedProto"),
    ("market_depth_exchanges_proto_buf", "marketDepthExchangesProto"),
    ("config_response_proto_buf", "configResponseProto"),
    ("update_config_response_proto_buf", "updateConfigResponseProto"),
)


#: What every member of the family on the client does, and on the wrapper.
_CLIENT_DOC = (
    "A request under the protobuf encoding, routed to the text request that "
    "carries the same intent: it states what its message states, and a field "
    "the message does not state falls to the text request's own default."
)
_WRAPPER_DOC = (
    "An answer under the protobuf encoding, which the reference wrapper "
    "carries as a stub: found by a program's super() call, and answering "
    "nothing — this client's engine delivers its answers under the text "
    "encoding."
)


def _shaped(ours, param, body, where, doc):
    """One method of the family, carrying the reference's parameter name.

    The name is what makes a keyword call work under either spelling: the
    pairing in the package reads it off the signature, and the signature is
    stated here because the body takes the argument however it arrives.
    """
    def method(self, *args, **given):
        if args:
            message = args[0]
        elif given:
            message = next(iter(given.values()))
        else:
            raise TypeError(f"{ours}() missing its one argument, {param!r}")
        return body(self, message)
    method.__name__ = ours
    method.__qualname__ = f"{where}.{ours}"
    method.__doc__ = doc
    method.__signature__ = inspect.Signature((
        inspect.Parameter("self", inspect.Parameter.POSITIONAL_ONLY),
        inspect.Parameter(param, inspect.Parameter.POSITIONAL_OR_KEYWORD)))
    return method


def use_proto_buf(self, msgId: int) -> bool:  # noqa: N803 — the reference's own name
    """Which encoding to speak: never the second one.

    The reference client answers True where the session's version supports
    the message. This client sends no message on any socket — its engine is
    in the same process — so there is no second encoding to speak and the
    answer is False, which sends a sample program down its text path.
    """
    return False


def install(surface: dict) -> None:
    """Put the family on the client and the stubs on the wrapper base.

    Run before the package pairs the two spellings, so the reference names
    are generated for what is put here as for any other method.
    """
    SURFACE.update(surface)
    client, wrapper = surface["EClient"], surface["EWrapper"]
    setattr(client, "use_proto_buf", use_proto_buf)
    for ours, param, route in _CLIENT:
        if hasattr(client, ours):
            continue  # the Rust surface already answers this one
        setattr(client, ours,
                _shaped(ours, param, _guarded(route), "EClient", _CLIENT_DOC))
    for ours, param in _WRAPPER:
        if hasattr(wrapper, ours):
            continue
        setattr(wrapper, ours, _shaped(ours, param, _stub, "EWrapper", _WRAPPER_DOC))


def _guarded(route):
    """A route behind the reference's own first line: a message nobody stated
    is nothing to send."""
    def body(self, message):
        if message is None:
            return None
        return route(self, _stated(message))
    return body


def _stub(self, message):
    """A stub on the wrapper base: found by a super() call, answering as
    every other stub here answers — nothing."""
    return None
