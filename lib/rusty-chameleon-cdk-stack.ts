import { Stack, StackProps } from "aws-cdk-lib";
import { Construct } from "constructs";
import * as cdk from "aws-cdk-lib";
import * as iam from "aws-cdk-lib/aws-iam";
import { config } from "dotenv";
import { RustFunction } from "cargo-lambda-cdk";

export class RustyChameleonCdkStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);
    config(); // config dotenv

    const pk = process.env.PUBLIC_KEY;
    if (!pk) {
      throw new Error("No Public Key Set in Env Vars");
    }

    const chameleon = new RustFunction(this, "RustyChameleonlambda", {
      manifestPath: "src/chameleon/Cargo.toml",
      bundling: {
        environment: {
          RUST_BACKTRACE: "1",
          PUBLIC_KEY: pk, // will throw invalidStringLength error if not set
          NASA_API_KEY: process.env.NASA_API_KEY || "",
        },
      },
    });

    chameleon.addToRolePolicy(
      new iam.PolicyStatement({
        actions: ["ssm:GetParameter"],
        resources: [
          "arn:aws:ssm:us-east-1:*:parameter/rusty-chameleon/public-key",
          "arn:aws:ssm:us-east-1:*:parameter/NASA_API_KEY",
        ],
        effect: cdk.aws_iam.Effect.ALLOW,
      }),
    );
  }
}
