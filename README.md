# Banyan

![izkp_Banyan_tree_with_complex_network_of_roots_wrapped_around_b_0f90a1b0-b3d0-4fd7-bd33-f36c781f0632](https://github.com/ipatka/banyan/assets/4401444/34129c1d-d9d6-401a-8529-c48ae74d55d4)



[![Crates.io](https://img.shields.io/crates/v/cedar-policy.svg)](https://crates.io/crates/cedar-policy)
[![docs.rs](https://img.shields.io/docsrs/cedar-policy)](https://docs.rs/cedar-policy/latest/cedar_policy/)
![nightly](https://github.com/cedar-policy/cedar/actions/workflows/nightly_build.yml/badge.svg)

This repository contains source code of the Rust crates that implement the Banyan fork of the [Cedar](https://www.cedarpolicy.com/) policy language.

Cedar is a language for writing and enforcing authorization policies in your applications. Using Cedar, you can write policies that specify your applications' fine-grained permissions. Your applications then authorize access requests by calling Cedar's authorization engine. Because Cedar policies are separate from application code, they can be independently authored, updated, analyzed, and audited. You can use Cedar's validator to check that Cedar policies are consistent with a declared schema which defines your application's authorization model.

Cedar is:
### Expressive
Cedar is a simple yet expressive language that is purpose-built to support authorization use cases for common authorization models such as RBAC and ABAC.
### Performant
Cedar is fast and scalable. The policy structure is designed to be indexed for quick retrieval and to support fast and scalable real-time evaluation, with bounded latency.
### Analyzable
Cedar is designed for analysis using Automated Reasoning. This enables analyzer tools capable of optimizing your policies and proving that your security model is what you believe it is.

Banyan is an extension of Cedar which adds Ethereum transaction context to enforce policies on the wallet or RPC  layer.

## All About Policies
### Anatomy of a Policy
#### Banyan Policy Language
Banyan is an adaptation of [AWS' Cedar Policy Language (CPL)](https://github.com/ipatka/banyan). Banyan adapts CPL for use in EVM transaction permissioning. You can get a firm understanding on the [cedar policy language in this tutorial.](https://www.cedarpolicy.com/en/tutorial/policy-structure) A CPL policy statement is made up of the following:
- The effect (forbid, permit)
- The scope (principal, action, resource)
- Conditions (unless, when)

A cedar file can contain a series of policy statements. Once a series of policy statements are executed, it will result in either "ALLOW" or "DENY". By default, everything not explicitly permitted results in "DENY", but this can be changed. See ["Setting Base Authorization"](https://hackmd.io/pZQrj_7HTMeBGP2PKMDljA#Setting-Base-Authorization). In instances where policies are conflicting, they will also return "DENY". 

Banyan makes use of decorators to extend the functionality of the Cedar Policy Language. A policy written in Banyan will follow the format in the example below.

```rust!
@name("Policy Statement 1 Name")
@message("This is the message returned when this policy fulfills it's action.")
@dependency("threatmodels.alerts")
@action("MFA")
permit(
    principal,
    action == 0xasldfakj,
    resource
) when {context.transaction.value.u256LessThan(u256("1234")) && threatmodels.alerts.time.u256LessThan(time.now()-time.hours(72))};
```
Contextual information is provided inside the json request.

#### Decorators
##### *@name*
This decorator is used to define and name a policy statement. This name will make an appearance in the Shield3 UI, messages sent to you regarding this policy, and can be used to reference the policy elsewhere.
```rust 
@name("Known phishing address")
```
##### *@message*
When the policy statement triggers a message, the user will receive this message. The message decorator was designed to be used as a way of warning against potentially risky situations.

```rust!
@message("Your transaction was blocked because the receiver of the transfer is a known phishing scammer. Please be sure to verify addresses before sending.")
```

##### *@action*
This is the action to carry out if the policy statement returns the relevant permission. The action can be defined as either Multi-Factor Authentication (MFA) or Notify. A DENY always results in block, however, ALLOW can result in either pass, notify, or MFA.

###### Block vs. Pass
Blocking means a transaction is not forwarded to the node service provider. If a transaction is blocked, a notification is sent to the transaction executor with the message defined in the *@message* decorator. If the transaction passes, then it is silently forwarded to the node service provider.



###### MFA
An MFA action is triggered when it's policy statement returns ALLOW. When MFA is triggered, the policy statement name, policy statement message, and transaction context will be sent to the transaction executor through their configured notifications, which can be found [here](url). 

The executor will then have the option to allow or deny the transaction. As shown previously, it's a good idea to set the message decorator as a warning, describing the risks coming with approving the transaction.
```rust
@action("MFA")
```

###### NOTIFY
When the notify action is triggered, the transaction will be broadcast as per usual, but notify the executor with the message provided from the message decorator.
```rust
@action("NOTIFY")
```
##### *@dependency*
```rust
@dependency("abcd-1234")
```
Any transaction context you'd like to reference can be found in your json request. Below is an example request, and a condition referencing it. In this case it returns a boolean to check if the network is mainnet ethereum. Any rich context you'd like to reference (information other than what is encoded in your transaction) must be defined as a dependency using the *@dependency* decorator. This rich context is then found in *entities.json*.

**request.json**
```json
{
  "principal": "Address::\"0xcfcdec1645234f521f29cb2bb0d57a539ba3bfae\"",
  "action": "Action::\"eoa\"",
  "resource": "Address::\"0x7c3250001bc0abeeef91f52e9054a9f951190132\"",
  "context": {
    "transaction": {
      "network": {
        "__entity": {
          "type": "Network",
          "id": "0x01"
        }
      },
      "data": "0x",
      "value": {
        "__expr": "u256(\"740048210\")"
      },
      "gasLimit": {
        "__expr": "u256(\"500000\")"
      }
    }
  }
}
```
**entities.json**
```json
[
  {
    "uid": {
      "type": "Address",
      "id": "0xd8a53b315823d8f8df8cb438c13ebe08af7c9ca9"
    },
    "attrs": {},
    "parents": []
  },
  {
    "uid": {
      "type": "Address",
      "id": "0x7a59293fe5fc36fdd762b4daeb07ba0873a3de44"
    },
    "attrs": {
      "groups": [
        {
          "__entity": {
            "type": "Group",
            "id": "1f033d2d-461a-4ce4-9026-5eb7efff5b4a"
          }
        }
      ]
    },
    "parents": []
  },
  {
    "uid": {
      "type": "Address",
      "id": "0xcfcdec1645234f521f29cb2bb0d57a539ba3bfae"
    },
    "attrs": {},
    "parents": []
  },
  {
    "uid": {
      "type": "Address",
      "id": "0x7c3250001bc0abeeef91f52e9054a9f951190132"
    },
    "attrs": {},
    "parents": []
  },
  {
    "uid": {
      "type": "Group",
      "id": "1f033d2d-461a-4ce4-9026-5eb7efff5b4a"
    },
    "attrs": {},
    "parents": []
  },
  {
    "uid": {
      "type": "Network",
      "id": "0x01"
    },
    "attrs": {
      "blockNumber": 18372931
    },
    "parents": []
  }
]

```

**policies.cedar**

```js
@name("Standard ERC20 TransferFrom Block Custom List")
@dependency("shared_addresses:block_list")
@message("Allow ERC20 transfer from unless blocked sender or recipient")
forbid(
  principal,
  action == Action::"0x23b872dd",
  resource
) when {
  (((((context["args"])["arg_0"]) has "groups") && ((((context["args"])["arg_0"])["groups"]).contains(Group::"block_list"))) || ((((context["args"])["arg_1"]) has "groups") && ((((context["args"])["arg_1"])["groups"]).contains(Group::"block_list")))) || ((principal has "groups") && ((principal["groups"]).contains(Group::"block_list")))
};
@name("Standard ERC20 TransferFrom Block Custom List")
@message("Allow ERC20 transfer from unless blocked sender or recipient")
@dependency("shared_addresses:block_list")
forbid(
  principal,
  action == Action::"0x23b872dd",
  resource
) when {
  (((((context["args"])["arg_0"]) has "groups") && ((((context["args"])["arg_0"])["groups"]).contains(Group::"block_list"))) || ((((context["args"])["arg_1"]) has "groups") && ((((context["args"])["arg_1"])["groups"]).contains(Group::"block_list")))) || ((principal has "groups") && ((principal["groups"]).contains(Group::"block_list")))
};
@dependency("shared_addresses:block_list")
@message("Allow transactions unless blocked sender or recipient or contract")
@name("Address or Contract Block Custom List")
permit(
  principal,
  action,
  resource
) when {
  !(((resource has "groups") && ((resource["groups"]).contains(Group::"block_list"))) || ((principal has "groups") && ((principal["groups"]).contains(Group::"block_list"))))
};
@dependency("shared_addresses:block_list")
@message("Allow ERC20 transfer unless blocked sender or recipient")
@name("Standard ERC20 Transfer Block Custom List")
forbid(
  principal,
  action == Action::"0xa9059cbb",
  resource
) when {
  ((((context["args"])["arg_0"]) has "groups") && ((((context["args"])["arg_0"])["groups"]).contains(Group::"block_list"))) || ((principal has "groups") && ((principal["groups"]).contains(Group::"block_list")))
};
@name("Non-Standard ERC20 Approve Block Custom List")
@message("Allow ERC20 approve unless blocked sender or recipient")
@dependency("shared_addresses:block_list")
forbid(
  principal,
  action == Action::"0x095ea7b3",
  resource
) when {
  ((((context["args"])["arg_0"]) has "groups") && ((((context["args"])["arg_0"])["groups"]).contains(Group::"block_list"))) || ((principal has "groups") && ((principal["groups"]).contains(Group::"block_list")))
};
@name("Standard ERC20 Approve Block Custom List")
@message("Allow ERC20 approve unless blocked sender or spender")
@dependency("shared_addresses:block_list")
forbid(
  principal,
  action == Action::"0x095ea7b3",
  resource
) when {
  ((((context["args"])["arg_0"]) has "groups") && ((((context["args"])["arg_0"])["groups"]).contains(Group::"block_list"))) || ((principal has "groups") && ((principal["groups"]).contains(Group::"block_list")))
};
@name("Non-Standard ERC20 Transfer Block Custom List")
@message("Allow ERC20 transfer unless blocked sender or recipient")
@dependency("shared_addresses:block_list")
forbid(
  principal,
  action == Action::"0xa9059cbb",
  resource
) when {
  ((((context["args"])["arg_0"]) has "groups") && ((((context["args"])["arg_0"])["groups"]).contains(Group::"block_list"))) || ((principal has "groups") && ((principal["groups"]).contains(Group::"block_list")))
};
```
