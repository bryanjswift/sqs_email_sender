import {CfnOutput, Construct, Duration, RemovalPolicy} from '@aws-cdk/core';
import {Alias, Key} from '@aws-cdk/aws-kms';
import {Queue as SQSQueue} from '@aws-cdk/aws-sqs';
import {Parameters} from './parameters';

type Props = Parameters;

export class QueueStack extends Construct {
  readonly encryptionKey: CfnOutput;
  readonly queueUrl: CfnOutput;

  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id);
    // Get the "Stage" being deployed
    const {stage} = props;
    // Set up SQS
    const queueEncryptionKey = new Key(this, 'EncryptionKey', {
      description:
        'The key used to encrypt messages sent to and received by the message queue.',
      enabled: true,
      enableKeyRotation: true,
      removalPolicy: RemovalPolicy.DESTROY,
    });
    const queueEncryptionAlias = new Alias(this, 'EncryptionKeyAlias', {
      aliasName: `alias/bryanjswift/email_service/${stage}/sqs`,
      targetKey: queueEncryptionKey,
    });
    queueEncryptionAlias.node.addDependency(queueEncryptionKey);
    const queue = new SQSQueue(this, 'Messages', {
      queueName: `email_queue_${stage}`,
      encryptionMasterKey: queueEncryptionAlias,
      retentionPeriod: Duration.days(4),
      visibilityTimeout: Duration.seconds(30),
    });
    queue.node.addDependency(queueEncryptionAlias);
    this.queueUrl = new CfnOutput(this, 'QueueUrl', {
      value: queue.queueUrl,
      description: 'URL of the Email Messages SQS Queue.',
    });
    this.encryptionKey = new CfnOutput(this, 'QueueEncryptionKey', {
      value: queueEncryptionAlias.keyArn,
      description:
        'ARN of the key used to encrypted Email Messages in SQS Queue.',
    });
  }
}
