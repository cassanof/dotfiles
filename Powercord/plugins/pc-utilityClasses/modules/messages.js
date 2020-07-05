const { findInReactTree } = require('powercord/util');
const { getModule } = require('powercord/webpack');
const { inject, uninject } = require('powercord/injector');

module.exports = async () => {
  const userStore = await getModule([ 'getCurrentUser' ]);
  const Message = await getModule(m => m.default && m.default.displayName === 'Message');
  inject('pc-utilitycls-messages', Message, 'default', (_, res) => {
    const msg = findInReactTree(res, n => n.message);
    if (!msg) {
      if (findInReactTree(res, n => n.className && n.className.startsWith('blockedSystemMessage'))) {
        res.props['data-is-blocked'] = 'true';
      }
      return res;
    }
    const { message } = msg;
    res.props['data-author-id'] = message.author.id;
    res.props['data-message-type'] = message.type;
    res.props['data-is-author-self'] = String(message.author.id === userStore.getCurrentUser().id);
    res.props['data-is-author-bot'] = String(message.author.bot);
    return res;
  });
  Message.default.displayName = 'Message';

  return async () => {
    uninject('pc-utilitycls-messages');
  };
};
