import {CfnOutput, Construct, RemovalPolicy} from '@aws-cdk/core';
import {
  AttributeType,
  BillingMode,
  Table as DynamoTable,
  TableEncryption,
} from '@aws-cdk/aws-dynamodb';
import {Alias, Key} from '@aws-cdk/aws-kms';
import {Parameters} from './parameters';

type Props = Parameters;

export class DatabaseStack extends Construct {
  readonly encryptionKey: CfnOutput;
  readonly tableName: CfnOutput;

  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id);
    // Get the "Stage" being deployed
    const {stage} = props;
    // Set up Dynamo
    const databaseEncryptionKey = new Key(this, 'EncryptionKey', {
      description: 'The key used to encrypt messages stored in the database.',
      enabled: true,
      enableKeyRotation: true,
      removalPolicy: RemovalPolicy.DESTROY,
    });
    const databaseEncryptionAlias = new Alias(this, 'EncryptionKeyAlias', {
      aliasName: `alias/bryanjswift/email_service/${stage}/db`,
      targetKey: databaseEncryptionKey,
    });
    databaseEncryptionAlias.node.addDependency(databaseEncryptionKey);
    const database = new DynamoTable(this, 'Messages', {
      tableName: `email_db_${stage}`,
      partitionKey: {
        name: 'EmailId',
        type: AttributeType.STRING,
      },
      billingMode: BillingMode.PAY_PER_REQUEST,
      encryption: TableEncryption.CUSTOMER_MANAGED,
      encryptionKey: databaseEncryptionAlias,
      removalPolicy: RemovalPolicy.DESTROY,
    });
    database.node.addDependency(databaseEncryptionAlias);
    this.tableName = new CfnOutput(this, 'DatabaseTable', {
      value: database.tableName,
      description: 'Name of the Dynamo Table storing message data.',
    });
    this.encryptionKey = new CfnOutput(this, 'DatabaseEncryptionKey', {
      value: databaseEncryptionAlias.keyArn,
      description:
        'ARN of the key used to encrypted Email Messages in Database.',
    });
  }
}
