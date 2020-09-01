import {expect, haveResource} from '@aws-cdk/assert';
import {App, Aws, Stack} from '@aws-cdk/core';
import {Alias, Key} from '@aws-cdk/aws-kms';
import test from 'tape';
import {KeyConstruct} from '../lib/key-construct';
import {AccountPrincipal} from '@aws-cdk/aws-iam';

test('KeyConstruct', (t) => {
  const app = new App();
  const stack = new Stack(app, 'MyTestStack');
  // WHEN
  const construct = new KeyConstruct(stack, 'KeyConstruct', {
    description: 'Test KeyConstruct description',
    serviceName: 'queue',
    stage: 'test',
  });
  // THEN
  t.is(
    construct.node.id,
    'KeyConstruct',
    'Stack name should match name from initialization'
  );
  t.is(
    construct.node.dependencies.length,
    1,
    'Should have list of dependencies containing each set of dependencies in the construct'
  );
  // It has an Alias -> Key dependency
  const aliasDep = construct.node.dependencies.find(
    (node) => node.source instanceof Alias
  );
  t.ok(aliasDep, 'There is a dependency starting with Alias');
  t.true(
    aliasDep?.target instanceof Key,
    'There is a dependency from Alias to Key'
  );
  // It has a KMS Key child
  try {
    // The `@aws-cdk/assert` library is written to be used with Jest. It throws
    // if the expectation fails.
    expect(stack).to(
      haveResource('AWS::KMS::Key', {
        Description: 'Test KeyConstruct description',
        Enabled: true,
        EnableKeyRotation: true,
      })
    );
  } catch (e) {
    t.fail('KMS Key exists in children', e instanceof Error ? e : undefined);
  }
  const accountPrincipal = new AccountPrincipal(Aws.ACCOUNT_ID);
  // Any grant
  const createGrant = construct.grant(accountPrincipal, 'kms:Create*');
  t.ok(createGrant, 'A grant for kms:Create* should be created.');
  t.true(
    createGrant.success,
    'The kms:Create* grant should have been successful'
  );
  const createGrantJson = createGrant.principalStatement?.toStatementJson();
  t.ok(createGrantJson, 'The kms:Create* grant provided a statement');
  t.is(
    createGrantJson.Action,
    'kms:Create*',
    'The kms:Create* grant has the correct action'
  );
  // Encrypt
  const encryptGrant = construct.grantEncrypt(accountPrincipal);
  t.true(encryptGrant.success, 'The kms:Encrypt grant should have succeeded');
  const encryptGrantJson = encryptGrant.principalStatement?.toStatementJson();
  t.ok(encryptGrantJson, 'The kms:Encrypt grant provided a statement');
  t.isEquivalent(
    encryptGrantJson.Action,
    ['kms:Encrypt', 'kms:ReEncrypt*', 'kms:GenerateDataKey*'],
    'The kms:Encrypt grant is for encryption'
  );
  // Decrypt
  const decryptGrant = construct.grantDecrypt(accountPrincipal);
  t.true(decryptGrant.success, 'The kms:Decrypt grant should have succeeded');
  const decryptGrantJson = decryptGrant.principalStatement?.toStatementJson();
  t.ok(decryptGrantJson, 'The kms:Decrypt grant provided a statement');
  t.is(
    decryptGrantJson.Action,
    'kms:Decrypt',
    'The kms:Decrypt grant is for decryption'
  );
  t.end();
});
