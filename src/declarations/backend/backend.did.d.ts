import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

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
export interface _SERVICE {
  'create_agent' : ActorMethod<[CreateAgentArgs], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
