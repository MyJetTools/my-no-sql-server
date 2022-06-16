# MY SERVICE BUS

## Changes
### 0.0.18
* BUG: Entities PartitionKey and RowKey now can not be uploaded with Null Values;
* Init procedure now skippes nulled values
* Bulk/InsertOrReplace - if we post payload with no entities -  TableUpdateMoment does not update;
* Added TcpSendTimeoutSec settings parameter