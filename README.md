# MY SERVICE BUS

## Changes
### 0.0.18
* BUG: Entities PartitionKey and RowKey now can not be uploaded with Null Values;
* Init procedure now skippes nulled values
* Bulk/InsertOrReplace - if we post payload with no entities -  TableUpdateMoment does not update;


### 0.0.19
* Improved loading performance;
* Added   settings parameter

### 0.0.20-rc01
* Sync to Readers now works in a separate thread
* Exposed sync to readers bytes to Prometheus reader by reader