import React from 'react';
import { Button, Card, Grid, Message, Modal, Form, Label } from 'semantic-ui-react';

import KittyAvatar from './KittyAvatar';
import { TxButton } from './substrate-lib/components';

// --- About Modal ---

const TransferModal = props => {
  const { kitty, is_owner, accountPair, setStatus } = props;
  if (!is_owner) {
      return null
  }
  const [open, setOpen] = React.useState(false);
  const [formValue, setFormValue] = React.useState({});

  const formChange = key => (ev, el) => {
    setFormValue(prev => ({ ...prev, [key]: el.value }));
  };

  const confirmAndClose = (unsub) => {
    unsub();
    setOpen(false);
  };

  return <Modal onClose={() => setOpen(false)} onOpen={() => setOpen(true)} open={open}
    trigger={<Button basic color='blue'>转让</Button>}>
    <Modal.Header>毛孩转让</Modal.Header>
    <Modal.Content><Form>
      <Form.Input fluid label='毛孩 ID' readOnly value={kitty.id}/>
      <Form.Input fluid label='转让对象' placeholder='对方地址' onChange={formChange('target')}/>
    </Form></Modal.Content>
    <Modal.Actions>
      <Button basic color='grey' onClick={() => setOpen(false)}>取消</Button>
      <TxButton
        accountPair={accountPair} label='确认转让' type='SIGNED-TX' setStatus={setStatus}
        onClick={confirmAndClose}
        attrs={{
          palletRpc: 'kittiesModule',
          callable: 'transfer',
          inputParams: [formValue.target, kitty.id],
          paramFields: [true, true]
        }}
      />
    </Modal.Actions>
  </Modal>;
};

// --- About Kitty Card ---
function stringToByte(str) {
    const bytes = [];
    let len, c;
    len = str.length;
    for(var i = 0; i < len; i++) {
        c = str.charCodeAt(i);
        if(c >= 0x010000 && c <= 0x10FFFF) {
            bytes.push(((c >> 18) & 0x07) | 0xF0);
            bytes.push(((c >> 12) & 0x3F) | 0x80);
            bytes.push(((c >> 6) & 0x3F) | 0x80);
            bytes.push((c & 0x3F) | 0x80);
        } else if(c >= 0x000800 && c <= 0x00FFFF) {
            bytes.push(((c >> 12) & 0x0F) | 0xE0);
            bytes.push(((c >> 6) & 0x3F) | 0x80);
            bytes.push((c & 0x3F) | 0x80);
        } else if(c >= 0x000080 && c <= 0x0007FF) {
            bytes.push(((c >> 6) & 0x1F) | 0xC0);
            bytes.push((c & 0x3F) | 0x80);
        } else {
            bytes.push(c & 0xFF);
        }
    }
    return bytes;
}
const KittyCard = props => {
  /*
    TODO: 加代码。这里会 UI 显示一张 `KittyCard` 是怎么样的。这里会用到：
    ```
    <KittyAvatar dna={dna} /> - 来描绘一只猫咪
    <TransferModal kitty={kitty} accountPair={accountPair} setStatus={setStatus}/> - 来作转让的弹出层
    ```
  */
  const { kitty, owner, accountPair, setStatus } = props;
  const dnaBytes = stringToByte(kitty.dna.toString().slice(2));
  const ownerAddress = "" + owner;

  let message = "";
  let isOwner = accountPair.address === ownerAddress;
  if (isOwner) {
      message += "[我的]";
  }
  return (
      <Grid.Column width={4}>
          <Card>
              <Card.Content>
                  <Card.Header>
                    {kitty.id}
                    <Card.Description>
                      {message}
                    </Card.Description>
                  </Card.Header>
                  <KittyAvatar dna={dnaBytes} />
                  <Card.Description>
                    主人: {ownerAddress}
                  </Card.Description>
                  <TransferModal kitty={kitty} is_owner={isOwner} accountPair={accountPair} setStatus={setStatus}/>
              </Card.Content>
          </Card>
      </Grid.Column>
  );
};

const KittyCards = props => {
  const { kitties, kittyOwners, kittyPrices, accountPair, setStatus } = props;
  return (
    <Grid>
      {kitties.map((kitty, index) => <KittyCard key={index} kitty={kitty} owner={kittyOwners[index]} price={kittyPrices[index]} accountPair={accountPair} setStatus={setStatus}/>)}
    </Grid>
  );
};

export default KittyCards;