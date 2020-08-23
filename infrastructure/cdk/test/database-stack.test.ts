import {anything, expect, haveResource} from '@aws-cdk/assert';
import {App, Stack} from '@aws-cdk/core';
import {Alias, Key} from '@aws-cdk/aws-kms';
import {
  AttributeType,
  BillingMode,
  Table as DynamoTable,
} from '@aws-cdk/aws-dynamodb';
import test from 'tape';
import {DatabaseStack} from '../lib/database-stack';

test('DatabaseStack', (t) => {
  const app = new App();
  const stack = new Stack(app, 'MyTestStack');
  // WHEN
  const construct = new DatabaseStack(stack, 'DatabaseConstruct', {
    stage: 'test',
  });
  // THEN
  t.is(
    construct.node.id,
    'DatabaseConstruct',
    'Stack name should match name from initialization'
  );
  t.is(
    construct.node.dependencies.length,
    2,
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
  // It has a Table -> Alias dependency
  const databaseDependency = construct.node.dependencies.find(
    (node) => node.source instanceof DynamoTable
  );
  t.ok(
    databaseDependency,
    'There is a dependency starting with the Dynamo DB Table'
  );
  t.true(
    databaseDependency?.target instanceof Alias,
    'There is a dependency from Dynamo DB Table to Alias'
  );
  // It has a Table child
  const database = construct.node.children.find(
    (node) => node instanceof DynamoTable
  );
  t.ok(database, 'Dynamo Table exists in children');
  try {
    // The `@aws-cdk/assert` library is written to be used with Jest. It throws
    // if the expectation fails.
    expect(stack).to(
      haveResource('AWS::DynamoDB::Table', {
        TableName: 'email_db_test',
        AttributeDefinitions: [
          {AttributeName: 'EmailId', AttributeType: AttributeType.STRING},
        ],
        KeySchema: [{AttributeName: 'EmailId', KeyType: 'HASH'}],
        BillingMode: BillingMode.PAY_PER_REQUEST,
        SSESpecification: {
          SSEEnabled: true,
          SSEType: 'KMS',
          KMSMasterKeyId: anything(),
        },
      })
    );
  } catch (e) {
    t.fail(
      'Dynamo Table exists in children',
      e instanceof Error ? e : undefined
    );
  }
  // It has a queue url output
  t.ok(construct.tableName, 'There is an output for Dynamo Table name');
  t.is(
    construct.tableName.node.id,
    'DatabaseTable',
    'At least according to its name'
  );
  t.end();
});
