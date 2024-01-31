import { NodejsFunction } from "aws-cdk-lib/aws-lambda-nodejs";
import { Construct } from "constructs";
import { Runtime } from "aws-cdk-lib/aws-lambda";
import { Table } from "aws-cdk-lib/aws-dynamodb";

interface MyLambdaParams {
    transactionsTable: Table
}

export class MyLambda extends NodejsFunction {
    constructor(scope: Construct, params: MyLambdaParams) {
        super(scope, 'MyLambda', {
            runtime: Runtime.PROVIDED_AL2, // Use the provided AWS Linux 2 runtime
            code: Code.fromAsset('target/lambda'), // Path to your compiled Rust binary
            handler: 'my_rust_binary', // Name of your Rust binary
            memorySize: 256,
            environment: {
                TRANSACTIONS_TABLE: params.transactionsTable.tableName
            }
        });
        params.transactionsTable.grantReadWriteData(this);
    }
}
