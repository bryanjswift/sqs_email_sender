import {Construct, RemovalPolicy} from '@aws-cdk/core';
import {
  AttributeType,
  BillingMode,
  Table as DynamoTable,
  TableEncryption,
} from '@aws-cdk/aws-dynamodb';
import {Grant, IGrantable} from '@aws-cdk/aws-iam';
import {KeyConstruct} from './key-construct';
import {Parameters} from './parameters';

type Props = Parameters;

export class DatabaseConstruct extends Construct {
  private readonly keyConstruct: KeyConstruct;
  readonly table: DynamoTable;

  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id);
    // Get the "Stage" being deployed
    const {stage} = props;
    // Set up Dynamo
    const databaseKeyConstruct = new KeyConstruct(this, 'EncryptionKey', {
      description: 'The key used to encrypt messages stored in the database.',
      serviceName: 'db',
      stage,
    });
    const databaseEncryptionKey = databaseKeyConstruct.key;
    const database = new DynamoTable(this, 'Messages', {
      tableName: `email_db_${stage}`,
      partitionKey: {
        name: 'EmailId',
        type: AttributeType.STRING,
      },
      billingMode: BillingMode.PAY_PER_REQUEST,
      encryption: TableEncryption.CUSTOMER_MANAGED,
      encryptionKey: databaseEncryptionKey,
      removalPolicy: RemovalPolicy.DESTROY,
    });
    database.node.addDependency(databaseEncryptionKey);
    // Set properties
    this.keyConstruct = databaseKeyConstruct;
    this.table = database;
  }

  grantDecrypt(grantee: IGrantable): Grant {
    return this.keyConstruct.grantDecrypt(grantee);
  }

  grantEncrypt(grantee: IGrantable): Grant {
    return this.keyConstruct.grantEncrypt(grantee);
  }

  grantReadData(grantee: IGrantable): Grant {
    return this.table.grantReadData(grantee);
  }

  grantReadWriteData(grantee: IGrantable): Grant {
    return this.table.grantReadWriteData(grantee);
  }

  grantWriteData(grantee: IGrantable): Grant {
    return this.table.grantReadWriteData(grantee);
  }
}
