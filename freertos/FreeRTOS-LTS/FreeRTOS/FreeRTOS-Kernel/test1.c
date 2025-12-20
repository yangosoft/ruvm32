#include "FreeRTOS.h"

#include <task.h>
#include <queue.h>
#include <timers.h>
#include <semphr.h>

/* Standard includes. */
// #include <stdio.h>


void vPortSetupTimerInterrupt(void);


void vPortSetupTimerInterrupt()
{
    // not implemented
    __asm__ volatile (
            "li a7, 67\n\t"   // llamada al sistema 64, write
            "ecall\n\t"
        );
}



/*-----------------------------------------------------------*/

static void exampleTask( void * parameters );
static void exampleTask2( void * parameters );

/*-----------------------------------------------------------*/

static void exampleTask( void * parameters )
{
    /* Unused parameters. */
    ( void ) parameters;

    for( ; ; )
    {
        /* Example Task Code */
        //vTaskDelay( 100 ); /* delay 100 ticks */
        __asm__ volatile (
            "li a7, 64\n\t"   // llamada al sistema 64, write
            "ecall\n\t"
        );
        taskYIELD();
    }
}

static void exampleTask2( void * parameters )
{
    /* Unused parameters. */
    ( void ) parameters;

    for( ; ; )
    {
        /* Example Task Code */
        //vTaskDelay( 100 ); /* delay 100 ticks */
        __asm__ volatile (
            "li a7, 65\n\t"   // llamada al sistema 64, write
            "ecall\n\t"
        );
        taskYIELD();
    }
}
/*-----------------------------------------------------------*/

void main( void )
{
    static StaticTask_t exampleTaskTCB;
    static StackType_t exampleTaskStack[ configMINIMAL_STACK_SIZE ];
    static StaticTask_t exampleTaskTCB2;
    static StackType_t exampleTaskStack2[ configMINIMAL_STACK_SIZE ];

    //( void ) printf( "Example FreeRTOS Project\n" );

    ( void ) xTaskCreateStatic( exampleTask,
                                "example",
                                configMINIMAL_STACK_SIZE,
                                NULL,
                                configMAX_PRIORITIES - 1U,
                                &( exampleTaskStack[ 0 ] ),
                                &( exampleTaskTCB ) );

    ( void ) xTaskCreateStatic( exampleTask2,
                                "example2",
                                configMINIMAL_STACK_SIZE,
                                NULL,
                                configMAX_PRIORITIES - 2U,
                                &( exampleTaskStack2[ 0 ] ),
                                &( exampleTaskTCB2 ) );

    /* Start the scheduler. */
    vTaskStartScheduler();

    for( ; ; )
    {
        /* Should not reach here. */
    }
}
/*-----------------------------------------------------------*/

#if ( configCHECK_FOR_STACK_OVERFLOW > 0 )

    void vApplicationStackOverflowHook( TaskHandle_t xTask,
                                        char * pcTaskName )
    {
        /* Check pcTaskName for the name of the offending task,
         * or pxCurrentTCB if pcTaskName has itself been corrupted. */
        ( void ) xTask;
        ( void ) pcTaskName;
    }

#endif /* #if ( configCHECK_FOR_STACK_OVERFLOW > 0 ) */
/*-----------------------------------------------------------*/
