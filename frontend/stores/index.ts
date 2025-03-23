import { defineStore } from 'pinia'

export const useIndex = defineStore('index', {
    state: () => ({
        darkMode: false,
        showAlert: false,
        alertVariant: 'success',
        alertMsg: '',
        secShowAlert: false,
        secAlertVariant: 'success',
        secAlertMsg: '',
        sseConnected: false,
        severityLevels: {
            DEBUG: 1,
            INFO: 2,
            WARN: 3,
            ERROR: 4,
        } as { [key: string]: number }
    }),

    getters: {},
    actions: {
        msgAlert(
                    variance: string, 
                    text: string, 
                    seconds: number = 3, 
                    sec_alert: boolean = false, 
                    sec_alert_variance: string = 'success',
                    sec_text: string = ''
                ) 
            {
                
            if (sec_alert) {
                this.secShowAlert = true
                this.secAlertMsg = sec_text
                this.secAlertVariant = sec_alert_variance
            }

            this.alertVariant = variance
            this.alertMsg = text
            this.showAlert = true

            setTimeout(() => {
            this.showAlert = false
            this.secShowAlert = false
            this.alertVariant = 'success'
            this.secAlertVariant = 'success'
            this.alertMsg = ''
            this.secAlertMsg = ''
            }, seconds * 1000)
        },
    },
})
