package com.mote.remote

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.graphics.Path
import android.os.Build
import android.view.accessibility.AccessibilityEvent

class TouchInputService : AccessibilityService() {
    
    companion object {
        var instance: TouchInputService? = null
            private set
    }

    override fun onServiceConnected() {
        super.onServiceConnected()
        instance = this
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        // Not used - we only need gesture dispatch
    }

    override fun onInterrupt() {
        // Not used
    }

    override fun onDestroy() {
        instance = null
        super.onDestroy()
    }

    fun tap(x: Float, y: Float) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            val path = Path()
            path.moveTo(x, y)
            
            val gestureBuilder = GestureDescription.Builder()
            gestureBuilder.addStroke(GestureDescription.StrokeDescription(path, 0, 50))
            
            dispatchGesture(gestureBuilder.build(), null, null)
        }
    }

    fun swipe(startX: Float, startY: Float, endX: Float, endY: Float, duration: Long = 300) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            val path = Path()
            path.moveTo(startX, startY)
            path.lineTo(endX, endY)
            
            val gestureBuilder = GestureDescription.Builder()
            gestureBuilder.addStroke(GestureDescription.StrokeDescription(path, 0, duration))
            
            dispatchGesture(gestureBuilder.build(), null, null)
        }
    }

    fun longPress(x: Float, y: Float) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            val path = Path()
            path.moveTo(x, y)
            
            val gestureBuilder = GestureDescription.Builder()
            gestureBuilder.addStroke(GestureDescription.StrokeDescription(path, 0, 1000))
            
            dispatchGesture(gestureBuilder.build(), null, null)
        }
    }
}
