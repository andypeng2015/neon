/*
 * Support functions for the compatibility macros in neon_pgversioncompat.h
 */
#include "postgres.h"

#include "funcapi.h"
#include "miscadmin.h"
#include "utils/tuplestore.h"

#include "neon_pgversioncompat.h"

#if PG_MAJORVERSION_NUM < 15
void
InitMaterializedSRF(FunctionCallInfo fcinfo, bits32 flags)
{
	ReturnSetInfo *rsinfo = (ReturnSetInfo *) fcinfo->resultinfo;
	Tuplestorestate *tupstore;
	TupleDesc	stored_tupdesc;

	if (get_call_result_type(fcinfo, NULL, &stored_tupdesc) != TYPEFUNC_COMPOSITE)
		elog(ERROR, "return type must be a row type");

	tupstore = tuplestore_begin_heap(false, false, work_mem);
	rsinfo->returnMode = SFRM_Materialize;
	rsinfo->setResult = tupstore;
	rsinfo->setDesc = stored_tupdesc;
}
#endif
