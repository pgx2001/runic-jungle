export const idlFactory = ({ IDL }) => {
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
  return IDL.Service({ 'create_agent' : IDL.Func([CreateAgentArgs], [], []) });
};
export const init = ({ IDL }) => { return []; };
