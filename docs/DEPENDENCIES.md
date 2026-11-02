# EH3D-PROJECT-PASSPORT: dependencies

One row per relation with another repository. The kind is conceptual, compile-time or run-time. The state of a contract is: not written, drafted, accepted, frozen or published. A repository existing does not mean that its contract is ready: this project does not implement or consume a contract that is not accepted.

| From | To | Kind | Contract | Contract state | Minimum version | Tested version | Compatibility test | Change owner | What it blocks | Migration |
|---|---|---|---|---|---|---|---|---|---|---|
| EH3D-PROJECT-PASSPORT | EH3D-COMMON | compile | project container and content hash | drafted | not set | not set | to be written with the first code | the owner of the contract | all of it | to be defined |
| EH3D-PROJECT-PASSPORT | EH3D-DESIGN-HISTORY | compile + runtime | semantic history | not written | not set | not set | to be written with the first code | the owner of the contract | the history section | to be defined |
| EH3D-PROJECT-PASSPORT | EH3D-PHOENIX-PCB | compile | project format (passport domain) | drafted | not set | not set | to be written with the first code | the owner of the contract | all of it | to be defined |
| EH3D-PROJECT-PASSPORT | EH3D-DESIGN-VERIFIER | conceptual | check results to record | not written | not set | not set | to be written with the first code | the owner of the contract | the results section | to be defined |
| EH3D-PROJECT-PASSPORT | EH3D-MANUFACTURING-PROFILE | conceptual | profile versions to record | not written | not set | not set | to be written with the first code | the owner of the contract | the profiles section | to be defined |
| EH3D-PROJECT-PASSPORT | EH3D-COMPONENT-INTELLIGENCE | conceptual | component provenance to record | not written | not set | not set | to be written with the first code | the owner of the contract | the components section | to be defined |
| EH3D-PROJECT-PASSPORT | EH3D-LAB-BRIDGE | conceptual | laboratory tests to record | not written | not set | not set | to be written with the first code | the owner of the contract | the laboratory section | to be defined |

Rules: repositories are built next to each other as sibling folders and each declares in its manifest the versions of the others that it needs, once those exist. A lower layer never depends on a higher one. The manifest field for build dependencies stays empty until code exists.
