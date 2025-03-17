export const idlFactory = ({ IDL }) => {
  const BitcoinNetwork = IDL.Variant({
    'mainnet' : IDL.Null,
    'regtest' : IDL.Null,
    'testnet' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'commission_receiver' : IDL.Opt(IDL.Principal),
    'commission' : IDL.Nat16,
    'creation_fee' : IDL.Nat64,
    'bitcoin_network' : BitcoinNetwork,
  });
  const AgentBy = IDL.Variant({ 'Id' : IDL.Nat, 'Name' : IDL.Text });
  const BuyArgs = IDL.Record({
    'id' : AgentBy,
    'amount_out_min' : IDL.Nat,
    'buy_exact_in' : IDL.Nat64,
  });
  const ChatArgs = IDL.Record({
    'agent' : AgentBy,
    'session_id' : IDL.Nat,
    'message' : IDL.Text,
  });
  const CreateAgentArgs = IDL.Record({
    'ticker' : IDL.Opt(IDL.Nat32),
    'twitter' : IDL.Opt(IDL.Text),
    'logo' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'description' : IDL.Text,
    'website' : IDL.Opt(IDL.Text),
    'discord' : IDL.Opt(IDL.Text),
    'openchat' : IDL.Opt(IDL.Text),
  });
  const AgentDetails = IDL.Record({
    'current_winner' : IDL.Opt(IDL.Principal),
    'market_cap' : IDL.Nat64,
    'ticker' : IDL.Nat32,
    'current_prize_pool' : IDL.Tuple(IDL.Nat64, IDL.Nat),
    'twitter' : IDL.Opt(IDL.Text),
    'runeid' : IDL.Text,
    'logo' : IDL.Opt(IDL.Text),
    'txns' : IDL.Tuple(IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)),
    'description' : IDL.Text,
    'created_at' : IDL.Nat64,
    'created_by' : IDL.Text,
    'website' : IDL.Opt(IDL.Text),
    'agent_name' : IDL.Text,
    'discord' : IDL.Opt(IDL.Text),
    'holders' : IDL.Nat32,
    'total_supply' : IDL.Nat,
    'openchat' : IDL.Opt(IDL.Text),
  });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const StreamingCallbackToken = IDL.Record({
    'chunk_index' : IDL.Nat32,
    'asset_id' : IDL.Nat,
    'content_encoding' : IDL.Text,
    'chunk_size' : IDL.Nat32,
  });
  const StreamingStrategy = IDL.Variant({
    'Callback' : IDL.Record({
      'token' : StreamingCallbackToken,
      'callback' : IDL.Func([], [], ['query']),
    }),
  });
  const HttpResponse = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'streaming_strategy' : IDL.Opt(StreamingStrategy),
    'status_code' : IDL.Nat16,
  });
  const LuckyDraw = IDL.Record({ 'id' : AgentBy, 'message' : IDL.Text });
  const SellArgs = IDL.Record({
    'id' : AgentBy,
    'token_amount' : IDL.Nat,
    'amount_collateral_min' : IDL.Nat64,
  });
  const WithdrawalType = IDL.Variant({
    'Rune' : IDL.Record({ 'runeid' : AgentBy, 'amount' : IDL.Nat }),
    'Bitcoin' : IDL.Record({ 'amount' : IDL.Nat64 }),
  });
  return IDL.Service({
    'buy' : IDL.Func([BuyArgs], [IDL.Nat], []),
    'chat' : IDL.Func([ChatArgs], [IDL.Text], []),
    'create_agent' : IDL.Func([CreateAgentArgs], [IDL.Nat], []),
    'create_chat_session' : IDL.Func([AgentBy], [IDL.Nat], []),
    'get_agent_of' : IDL.Func([AgentBy], [IDL.Opt(AgentDetails)], ['query']),
    'get_agents' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Nat, AgentDetails))],
        ['query'],
      ),
    'get_balances' : IDL.Func([], [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat))], []),
    'get_bitcoin_balance' : IDL.Func([], [IDL.Nat64], []),
    'get_deposit_address' : IDL.Func([], [IDL.Text], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'lucky_draw' : IDL.Func([LuckyDraw], [IDL.Text], []),
    'sell' : IDL.Func([SellArgs], [IDL.Nat], []),
    'withdraw' : IDL.Func([IDL.Text, WithdrawalType], [IDL.Nat], []),
  });
};
export const init = ({ IDL }) => {
  const BitcoinNetwork = IDL.Variant({
    'mainnet' : IDL.Null,
    'regtest' : IDL.Null,
    'testnet' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'commission_receiver' : IDL.Opt(IDL.Principal),
    'commission' : IDL.Nat16,
    'creation_fee' : IDL.Nat64,
    'bitcoin_network' : BitcoinNetwork,
  });
  return [InitArgs];
};
