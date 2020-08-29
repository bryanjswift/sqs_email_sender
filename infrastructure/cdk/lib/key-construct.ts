import {Construct, RemovalPolicy} from '@aws-cdk/core';
import {Grant, IGrantable} from '@aws-cdk/aws-iam';
import {Alias, Key} from '@aws-cdk/aws-kms';
import {Parameters} from './parameters';

interface Props extends Parameters {
  description: string;
  serviceName: string;
}

export class KeyConstruct extends Construct {
  readonly key: Alias;

  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id);
    // Get the "Stage" being deployed
    const {description, serviceName, stage} = props;
    // Set up SQS
    const encryptionKey = new Key(this, 'EncryptionKey', {
      description,
      enabled: true,
      enableKeyRotation: true,
      removalPolicy: RemovalPolicy.DESTROY,
    });
    const encryptionAlias = new Alias(this, 'EncryptionKeyAlias', {
      aliasName: `alias/bryanjswift/email_service/${stage}/${serviceName}`,
      targetKey: encryptionKey,
    });
    encryptionAlias.node.addDependency(encryptionKey);
    // Store values
    this.key = encryptionAlias;
  }

  grant(grantee: IGrantable, ...actions: string[]): Grant {
    return this.key.grant(grantee, ...actions);
  }

  grantDecrypt(grantee: IGrantable): Grant {
    return this.key.grantDecrypt(grantee);
  }

  grantEncrypt(grantee: IGrantable): Grant {
    return this.key.grantEncrypt(grantee);
  }
}
