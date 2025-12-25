

// DYNAMIC_SWITCHING with Environment Management
// Implement Environment Management


REFERENCE_TABLE = {
    
    RABBIT_LOOKUP_TABLE : {

        ACTIVE_IDS : ["IDENT_ID_39"],

        USAGE : {
            "IDENT_ID_39" : 148
        }
    },
    
    TURTLE_LOOKUP_TABLE : {
        INACTIVE_IDS : [],

        USAGE : {
            
        }
    },

    
    REFERENCE_PAGES : {

        PAGE_ID_39 : {
            
            REFERENCE: {
            	
            	VALUE        : 10,
            	REF_COUNT    : 3,
            	ORIGIN_LINE  : 39,
            	HEAP_ALLOC   : false,
            	STACK_ALLOC  : true,
		
            	REFERENCE_CACHE : [
                	"x", "y", "z"
				],
		
		
            	ACCESS_FREQUENCY : 148
            	LAST_ACCESS_TIME : 9349234
            },
            
            ACCESSIBILITY: {
            
            	VALUE : 20,
            	ACCESSIBILITY_COUNT : 4,
            	ORIGIN_LINE : 54,
            	HEAP_ALLOC   : false,
            	STACK_ALLOC  : true,
            	
            	ACCESSIBILITY_CACHE : {
                	"a", "b", "c", "d"
            	},
            	ACCESS_FREQUENCY : 148
            	LAST_ACCESS_TIME : 9349234
            }
        }
    }
}


// REFERENCING
x = 10
y = x
z = y

// ACCESSIBILITY

a = 20
b = 20
c = 20
d = 20

