import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type AgentBy = { 'Id' : bigint } |
  { 'Name' : string };
export interface AgentDetails {
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
export interface BuyArgs { 'id' : AgentBy, 'min_amount_out' : bigint }
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
export interface LuckyDraw { 'id' : AgentBy, 'message' : string }
export interface SellArgs { 'id' : AgentBy, 'min_amount_out' : bigint }
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
  'buy' : ActorMethod<[BuyArgs], undefined>,
  'chat' : ActorMethod<[ChatArgs], undefined>,
  'create_agent' : ActorMethod<[CreateAgentArgs], bigint>,
  'get_agent_of' : ActorMethod<[AgentBy], [] | [AgentDetails]>,
  'get_agents' : ActorMethod<[], Array<[bigint, AgentDetails]>>,
  'get_deposit_address' : ActorMethod<[], string>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'lucky_draw' : ActorMethod<[LuckyDraw], undefined>,
  'sell' : ActorMethod<[SellArgs], undefined>,
  'withdraw' : ActorMethod<[string, WithdrawalType], bigint>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
