"""Every callback this client can fire is one a subclass can override.

The list is written out rather than read off the class, so it cannot shrink to
match what happens to be there. A callback that loses its name, or is spelled
one way in dispatch and another on the base class, reaches the caller's code
never and says nothing about it — which is the failure this whole surface is
built to avoid.

81 callbacks, the count the reference table publishes.
"""

from decimal import Decimal

from ibkr_dx import EWrapper

CALLBACKS = [
    "connect_ack", "connection_closed", "next_valid_id", "managed_accounts",
    "error", "current_time", "tick_price", "tick_size", "tick_string",
    "tick_generic", "tick_snapshot_end", "market_data_type",
    "tick_req_params", "order_status", "open_order", "open_order_end",
    "order_bound", "exec_details", "exec_details_end",
    "commission_and_fees_report", "update_account_value",
    "update_portfolio", "update_account_time", "account_download_end",
    "account_summary", "account_summary_end", "position", "position_end",
    "pnl", "pnl_single", "position_multi", "position_multi_end",
    "account_update_multi", "account_update_multi_end", "contract_details",
    "contract_details_end", "bond_contract_details", "symbol_samples",
    "historical_data", "historical_data_end", "historical_data_update",
    "head_timestamp", "historical_ticks", "historical_ticks_bid_ask",
    "historical_ticks_last", "histogram_data", "historical_schedule",
    "update_mkt_depth", "update_mkt_depth_l2", "mkt_depth_exchanges",
    "tick_by_tick_all_last", "tick_by_tick_bid_ask",
    "tick_by_tick_mid_point", "scanner_data", "scanner_data_end",
    "scanner_parameters", "news_providers", "news_article",
    "historical_news", "historical_news_end", "tick_news",
    "update_news_bulletin", "real_time_bar", "fundamental_data",
    "market_rule", "completed_order", "completed_orders_end",
    "tick_option_computation", "security_definition_option_parameter",
    "security_definition_option_parameter_end", "smart_components",
    "soft_dollar_tiers", "family_codes", "user_info", "receive_fa",
    "replace_fa_end", "display_group_list", "display_group_updated",
    "delta_neutral_validation", "wsh_meta_data", "wsh_event_data",
]


def test_the_count_is_the_published_one():
    assert len(CALLBACKS) == 81


def test_every_callback_is_there_to_override():
    missing = [name for name in CALLBACKS if not hasattr(EWrapper, name)]
    assert not missing, f"a subclass cannot override: {missing}"


def test_every_one_of_them_is_callable():
    """A name resolving to something that is not a method is not a callback."""
    unusable = [name for name in CALLBACKS if not callable(getattr(EWrapper, name, None))]
    assert not unusable, f"named and not callable: {unusable}"


#: Every shape where the reference client's word for a parameter is not this
#: client's word for it, plus the three methods both clients spell the same,
#: each called with the reference's own keywords: what a tee or relay
#: forwarding through the base sends, and what a moved program's
#: `super().tickPrice(reqId=...)` sends. A keyword the base refuses raises
#: inside the delivered callback, and a raise there closes the session.
REFERENCE_KEYWORD_SHAPES = [
    ("error", dict(reqId=1, errorTime=0, errorCode=501, errorString="Already connected.", advancedOrderRejectJson="")),
    ("updateAccountValue", dict(key="k", val="v", currency="USD", accountName="U1")),
    ("accountDownloadEnd", dict(accountName="U1")),
    ("position", dict(account="U1", contract=None, position=Decimal(1), avgCost=1.0)),
    ("pnl", dict(reqId=1, dailyPnL=1.0, unrealizedPnL=2.0, realizedPnL=3.0)),
    ("realtimeBar", dict(reqId=1, time=0, open_=1.0, high=2.0, low=0.5, close=1.5, volume=Decimal(10), wap=1.2, count=3)),
    ("receiveFA", dict(faData=1, cxml="<x/>")),
    ("updateNewsBulletin", dict(msgId=1, msgType=1, newsMessage="m", originExch="NYSE")),
    ("newsArticle", dict(requestId=1, articleType=1, articleText="t")),
    ("historicalNews", dict(requestId=1, time="t", providerCode="p", articleId="a", headline="h")),
    ("historicalNewsEnd", dict(requestId=1, hasMore=False)),
    ("tickEFP", dict(reqId=1, tickType=1, basisPoints=1.0, formattedBasisPoints="1", totalDividends=0.5, holdDays=1, futureLastTradeDate="d", dividendImpact=0.1, dividendsToLastTradeDate=0.2)),
    ("tickReqParams", dict(tickerId=1, minTick=0.01, bboExchange="X", snapshotPermissions=0)),
    ("verifyAndAuthMessageAPI", dict(apiData="a", xyzChallange="c")),
    ("orderStatus", dict(orderId=1, status="Filled", filled=Decimal(1), remaining=Decimal(0), avgFillPrice=1.0, permId=7, parentId=0, lastFillPrice=1.0, clientId=0, whyHeld="", mktCapPrice=0.0)),
    ("orderBound", dict(permId=1, clientId=0, orderId=2)),
    ("nextValidId", dict(orderId=1)),
]


def test_every_callback_answers_to_the_reference_keywords():
    w = EWrapper()
    for name, kwargs in REFERENCE_KEYWORD_SHAPES:
        getattr(w, name)(**kwargs)  # a refusal here is the defect, not an assert
