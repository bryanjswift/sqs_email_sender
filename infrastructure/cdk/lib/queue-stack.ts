import {Construct, Duration} from '@aws-cdk/core';
import {Grant, IGrantable} from '@aws-cdk/aws-iam';
import {Queue as SQSQueue} from '@aws-cdk/aws-sqs';
import {KeyConstruct} from './key-construct';
import {Parameters} from './parameters';

type Props = Parameters;

export class QueueStack extends Construct {
  private readonly keyConstruct: KeyConstruct;
  readonly queue: SQSQueue;

  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id);
    // Get the "Stage" being deployed
    const {stage} = props;
    // Set up SQS
    const queueKeyConstruct = new KeyConstruct(this, 'EncryptionKey', {
      description:
        'The key used to encrypt messages sent to and received by the message queue.',
      serviceName: 'queue',
      stage,
    });
    const queueEncryptionKey = queueKeyConstruct.key;
    const queue = new SQSQueue(this, 'Messages', {
      queueName: `email_queue_${stage}`,
      encryptionMasterKey: queueEncryptionKey,
      retentionPeriod: Duration.days(4),
      visibilityTimeout: Duration.seconds(30),
    });
    queue.node.addDependency(queueEncryptionKey);
    this.queue = queue;
    this.keyConstruct = queueKeyConstruct;
  }

  grant(grantee: IGrantable, ...actions: string[]): Grant {
    return this.keyConstruct.grant(grantee, ...actions);
  }

  grantDecrypt(grantee: IGrantable): Grant {
    return this.keyConstruct.grantDecrypt(grantee);
  }

  grantEncrypt(grantee: IGrantable): Grant {
    return this.keyConstruct.grantEncrypt(grantee);
  }
}
