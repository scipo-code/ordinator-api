struct ActivityAgent {
    order: Int,
    activity: Union{Int, Missing},
    sch_start: DateTime,
    sch_date: DateTime,
    period: Period,
    assigned: Vector{Int},
}
