import polars as pl

# We should load in the WorkOrder data and then join the data correctly afterwards.
secondary_locations_dtypes = {
    "PM_Object_Sorting" : pl.Float64
}


mid_work_orders = pl.read_csv("mid_work_orders.csv").lazy()

mid_work_orders = mid_work_orders.with_columns("WO_Operation_ID").cast({"WO_Operation_ID" : pl.Int64})

mid_functional_locations = pl.read_csv("mid_functional_locations.csv").lazy()
# mid_secondary_locations = pl.read_csv("mid_secondary_locations.csv", schema_overrides=secondary_locations_dtypes).lazy()
mid_work_center = pl.read_csv("mid_work_center.csv").lazy()
mid_work_operations = pl.read_csv("mid_work_operations.csv").lazy()
mid_work_orders_status = pl.read_csv("mid_work_orders_status.csv").lazy()


# Aggregate the status codes by work order
mid_work_orders_status = mid_work_orders_status.group_by("WO_Object_Number").agg(
    pl.concat_str("WO_I_Status_Code", ignore_nulls=True),
    pl.concat_str("WO_E_Status_Code", ignore_nulls=True))
mid_work_orders_status = mid_work_orders_status.with_columns(
    pl.col("WO_I_Status_Code").list.join(separator=" "),
    pl.col("WO_E_Status_Code").list.join(separator=" "))

# Remove the rows that does not contain the "REL" (Released) status code

print(mid_work_orders_status)
mid_work_orders_status = mid_work_orders_status.filter(pl.col("WO_I_Status_Code").str.contains(r'REL'))
print(mid_work_orders_status)

# mid_work_orders_status = mid_work_orders_status.filter(pl.col("WO_I_Status_Code")
    

combined_df = mid_work_orders_status.join(mid_work_orders, left_on = "WO_Object_Number", right_on = "WO_Status_ID", how = "left")
combined_df = combined_df.join(mid_work_operations, left_on = "WO_Operation_ID", right_on = "OPR_Routing_Number", how = "left", suffix = "_operations")

print(combined_df.collect())

combined_df = combined_df.join(mid_work_center, left_on = "OPR_WBS_ID", right_on = "WBS_ID", how = "left", suffix = "_work_center")
combined_df = combined_df.join(mid_functional_locations, left_on = "WO_Functional_Location_Number", right_on = "FLOC_Technical_ID", how = "left", suffix = "_functional_location")
print(combined_df.collect())
combined_df = combined_df.join(mid_work_center, left_on = "WO_WBS_ID", right_on = "WBS_ID", how = "left")
print(combined_df.collect())

combined_df.collect().write_excel("dlap_data_extract")
