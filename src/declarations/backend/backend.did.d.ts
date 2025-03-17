import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type AgentBy = { 'Id' : bigint } |
  { 'Name' : string };
export interface AgentDetails {
  'current_winner' : [] | [Principal],
  'market_cap' : bigint,
  'ticker' : number,
  'current_prize_pool' : [bigint, bigint],
  'twitter' : [] | [string],
  'runeid' : string,
  'logo' : [] | [string],
  'txns' : [[] | [string], [] | [string]],
  'description' : string,
  'created_at' : bigint,
  'created_by' : string,
  'website' : [] | [string],
  'agent_name' : string,
  'discord' : [] | [string],
  'holders' : number,
  'total_supply' : bigint,
  'openchat' : [] | [string],
}
export type BitcoinNetwork = { 'mainnet' : null } |
  { 'regtest' : null } |
  { 'testnet' : null };
export interface BuyArgs {
  'id' : AgentBy,
  'amount_out_min' : bigint,
  'buy_exact_in' : bigint,
}
export interface ChatArgs {
  'agent' : AgentBy,
  'session_id' : [] | [bigint],
  'message' : string,
}
export interface CreateAgentArgs {
  'ticker' : [] | [number],
  'twitter' : [] | [string],
  'logo' : [] | [string],
  'name' : string,
  'description' : string,
  'website' : [] | [string],
  'discord' : [] | [string],
  'openchat' : [] | [string],
}
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
  'streaming_strategy' : [] | [StreamingStrategy],
  'status_code' : number,
}
export interface InitArgs {
  'commission_receiver' : [] | [Principal],
  'commission' : number,
  'creation_fee' : bigint,
  'bitcoin_network' : BitcoinNetwork,
}
export interface LuckyDraw { 'id' : AgentBy, 'message' : string }
export interface SellArgs {
  'id' : AgentBy,
  'token_amount' : bigint,
  'amount_collateral_min' : bigint,
}
export interface StreamingCallbackToken {
  'chunk_index' : number,
  'asset_id' : bigint,
  'content_encoding' : string,
  'chunk_size' : number,
}
export type StreamingStrategy = {
    'Callback' : {
      'token' : StreamingCallbackToken,
      'callback' : [Principal, string],
    }
  };
export type WithdrawalType = {
    'Rune' : { 'runeid' : AgentBy, 'amount' : bigint }
  } |
  { 'Bitcoin' : { 'amount' : bigint } };
export interface _SERVICE {
  'buy' : ActorMethod<[BuyArgs], bigint>,
  'chat' : ActorMethod<[ChatArgs], string>,
  'create_agent' : ActorMethod<[CreateAgentArgs], bigint>,
  'get_agent_of' : ActorMethod<[AgentBy], [] | [AgentDetails]>,
  'get_agents' : ActorMethod<[], Array<[bigint, AgentDetails]>>,
  'get_balances' : ActorMethod<[], Array<[string, bigint]>>,
  'get_deposit_address' : ActorMethod<[], string>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'lucky_draw' : ActorMethod<[LuckyDraw], string>,
  'sell' : ActorMethod<[SellArgs], bigint>,
  'withdraw' : ActorMethod<[string, WithdrawalType], bigint>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
